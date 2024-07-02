use scrypto::prelude::*;

//r and data
#[derive(ScryptoSbor, Clone)]
pub struct Loan {
    lender: ComponentAddress,
    borrower: ComponentAddress,
    amount: Decimal,
    interest_rate_per_month: Decimal,
    duration_in_months: u8,
    start_time: u64,
    is_repaid: bool,
}

#[blueprint]
mod lending_borrowing {
    struct LendingBorrowing {
        loans: HashMap<u128, Loan>, // loan_storage
        loan_count: u128,
        lender_vault: HashMap<ComponentAddress, Vault>,
    }

    impl LendingBorrowing {
        pub fn instantiate_lending_borrowing() -> Global<LendingBorrowing> {
            Self {
                loans: HashMap::new(),
                loan_count: 0,
                lender_vault: HashMap::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn lend_tokens(
            &mut self,
            borrower: ComponentAddress,
            amount: Decimal,
            interest_rate_per_month: Decimal, // 0.05 means 5% interest
            duration_in_months: u8,
            lender_address: ComponentAddress,
            lender_tokens: Bucket, // resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:1000
        ) -> String {
            let loan_id = self.loan_count;

            let loan = Loan {
                lender: lender_address,
                borrower,
                amount,
                interest_rate_per_month,
                duration_in_months,
                start_time: Runtime::current_epoch().number(), // start time
                is_repaid: false,
            };

            self.loans.insert(loan_id, loan);

            self.loan_count += 1;

            // why there is a need for this ?
            let lender_vault = self
                .lender_vault
                .entry(lender_address)
                .or_insert_with(|| Vault::new(lender_tokens.resource_address()));

            lender_vault.put(lender_tokens);

            info!(
                "Loan created with ID: {}. Lender: {:?}, Borrower: {:?}, Amount: {}, Interest Rate: {}, Duration: {} months",
                loan_id, lender_address, borrower, amount, interest_rate_per_month, duration_in_months
            );

            format!(
                "loan is granted successfully to the borrower : {:?} and here is the loan id : {}",
                borrower, loan_id
            )
        }

        // Function to repay a loan
        pub fn repay_loan(&mut self, loan_id: u128, mut payment: Bucket) -> (Bucket, Bucket) {
            let loan = self.loans.get_mut(&loan_id).expect("Loan not found");

            assert!(!loan.is_repaid, "The loan is already repaid");

            let duration_in_months = loan.duration_in_months as u64;

            let interest_rate = loan.interest_rate_per_month;

            let amount = loan.amount;

            let current_time = Runtime::current_epoch().number();

            let time_elapsed = current_time - loan.start_time; // Calculate the time elapsed since the loan started

            // It is being checked that how many months have been passed
            let months_elapsed = time_elapsed as u32 / (30 * duration_in_months) as u32; // Assuming `time_elapsed` gives you days

            info!(
                "months_elapsed : {}",
                 months_elapsed
            );

            let accrued_interest = amount * (Decimal::one() + interest_rate).0.pow(months_elapsed) - amount;

            // Calculate the accrued interest
            // let accrued_interest = amount
            //     * (Decimal::one() + interest_rate)
            //         .0
            //         .pow(time_elapsed as u32 / (30 * duration_in_months) as u32)
            //     - amount;

            info!(
                "This is the accrued interest you will be paying : {}",
                accrued_interest
            );

            let total_repayment = amount + accrued_interest;

            info!(
                "This much amount need to be repaid (amount + accrued interest) : {}",
                total_repayment
            );

            assert!(
                payment.amount() >= total_repayment,
                "Insufficient payment to repay the loan"
            );

            // Update the lender's vault with the payment
            let lender_vault = self
                .lender_vault
                .get_mut(&loan.lender)
                .expect("Lender vault not found");

            lender_vault.put(payment.take(total_repayment));

            loan.is_repaid = true;

            (payment, lender_vault.take(0)) // Return the remaining payment and an empty bucket
        }

        // Function to get loan details by loan ID
        pub fn get_loan_details(&self, loan_id: u128) -> Option<Loan> {
            let res = self.loans.get(&loan_id).cloned();
            res
        }
    }
}

// resim reset
// resim new-account
// resim publish .
// resim call-function package_sim1pk3cmat8st4ja2ms8mjqy2e9ptk8y6cx40v4qnfrkgnxcp2krkpr92 LendingBorrowing instantiate_lending_borrowing
// component_sim1cqyavav59dl55jur4eyxqz9wqyjycp2aua9dzduflfeefrfl5sdpuy

// lend
// resim call-method component_sim1cqyavav59dl55jur4eyxqz9wqyjycp2aua9dzduflfeefrfl5sdpuy lend_tokens account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma 1000 100 1 account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:1000

// repay_loan
// resim call-method component_sim1cqyavav59dl55jur4eyxqz9wqyjycp2aua9dzduflfeefrfl5sdpuy repay_loan 0 resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:1000

// loan_details
// resim call-method component_sim1cqyavav59dl55jur4eyxqz9wqyjycp2aua9dzduflfeefrfl5sdpuy get_loan_details 0

// u8, u64, u128
