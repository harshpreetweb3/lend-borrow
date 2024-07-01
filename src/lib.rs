use scrypto::prelude::*;

//r and data
#[derive(ScryptoSbor)]
#[derive(Clone)]
pub struct Loan {
    lender: ComponentAddress,
    borrower: ComponentAddress,
    amount: Decimal,
    interest_rate: Decimal,
    duration_in_months: u8,
    start_time: u64,
    is_repaid: bool,
}

#[blueprint]
mod lending_borrowing {
    struct LendingBorrowing {
        loans: HashMap<u128, Loan>,
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

        // resim call-method component_sim1cqyavav59dl55jur4eyxqz9wqyjycp2aua9dzduflfeefrfl5sdpuy account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma 1000 100 1 account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:1000
        //account_sim1c9yeaya6pehau0fn7vgavuggeev64gahsh05dauae2uu25njk224xz 1000 100 1 account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3:100
        pub fn lend_tokens(
            &mut self,
            borrower: ComponentAddress,
            amount: Decimal,
            interest_rate: Decimal,
            duration_in_months: u8,
            lender_address: ComponentAddress,
            lender_tokens: Bucket,
        ) -> u128 {

            let loan_id = self.loan_count;

            let loan = Loan {
                lender: lender_address,
                borrower,
                amount,
                interest_rate,
                duration_in_months,
                start_time: Runtime::current_epoch().number(),
                is_repaid: false,
            };

            self.loans.insert(loan_id, loan);

            self.loan_count += 1;

            let lender_vault = self
                .lender_vault
                .entry(lender_address)
                .or_insert_with(|| Vault::new(lender_tokens.resource_address()));

            lender_vault.put(lender_tokens);

            info!(
                "Loan created with ID: {}. Lender: {:?}, Borrower: {:?}, Amount: {}, Interest Rate: {}, Duration: {} months",
                loan_id, lender_address, borrower, amount, interest_rate, duration_in_months
            );

            loan_id
        }

         // Function to repay a loan
         pub fn repay_loan(&mut self, loan_id: u128, mut payment: Bucket) -> (Bucket, Bucket) {
            let loan = self.loans.get_mut(&loan_id).expect("Loan not found"); // Get the loan from the HashMap

            assert!(
                !loan.is_repaid,
                "The loan is already repaid"
            );

            let duration_in_months = loan.duration_in_months as u64; // Get the loan duration in months
            let interest_rate = loan.interest_rate; // Get the interest rate
            let amount = loan.amount;               // Get the loan amount
            let current_time = Runtime::current_epoch().number();
            let time_elapsed = current_time - loan.start_time; // Calculate the time elapsed since the loan started

            // Calculate the accrued interest
            let accrued_interest = amount
                * (Decimal::one() + interest_rate).0.pow(time_elapsed as u32 / (30 * duration_in_months) as u32)
                - amount;

            let total_repayment = amount + accrued_interest; // Calculate the total repayment amount

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

            loan.is_repaid = true; // Mark the loan as repaid

            (payment, lender_vault.take(0)) // Return the remaining payment and an empty bucket
        }

        // Function to get loan details by loan ID
        pub fn get_loan_details(&self, loan_id: u128) -> Option<Loan> {
            let res = self.loans.get(&loan_id).cloned(); // Return a clone of the loan details if found
            res
        }
    }
}
