#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[ink::contract]
pub mod contract {
    use ink::env::call::Selector;
    use ink::prelude::vec::Vec;
    use scale::Encode;
    use scheduler_extension::*;
    use sp_weights::Weight;

    #[ink(storage)]
    #[derive(Default)]
    pub struct Scheduler {
        value: u32,
    }

    impl Scheduler {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { value: 0 }
        }

        #[ink(message)]
        pub fn schedule(&mut self, value: u32) -> Result<(), SchedulerError> {
            let block_number = self.env().block_number();

            let mut data = Vec::new();
            let mut selector: Vec<u8> = Selector::new(ink::selector_bytes!("set_value"))
                .to_bytes()
                .to_vec();
            data.append(&mut selector);
            data.append(&mut value.encode());
            // data: 0xc62982150a000000 if value = 10

            let call = ContractCallInput {
                dest: self.env().account_id(),
                data,
                // got from contracts-UI
                gas_limit: Weight::from_parts(3951114240u64, 629760u64),
                storage_deposit_limit: None,
                value: 0,
                max_weight: 1_000_000_000_000u64,
            };
            SchedulerExtension::schedule(Origin::Address, block_number + 3, None, 0, call)
        }

        #[ink(message)]
        pub fn set_value(&mut self, value: u32) {
            self.value = value;
        }

        #[ink(message)]
        pub fn get_value(&self) -> u32 {
            self.value
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::*;
    use ink_e2e::build_message;
    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn schedule_set_value_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // given
        let constructor = Scheduler::new();
        let contract_acc_id = client
            .instantiate("scheduler_example", &ink_e2e::alice(), constructor, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;

        // when
        let schedule =
            build_message::<Scheduler>(contract_acc_id.clone()).call(|scheduler| {
                scheduler.schedule(
                    10,
                )
            });
        advance_block(5);

        // then
        let get_value_msg = build_message::<Scheduler>(contract_acc_id.clone())
            .call(|scheduler| scheduler.get_value());
        let get_value_res = client
            .call_dry_run(&ink_e2e::alice(), &get_value_msg, 0, None)
            .await;
        assert_eq!(
            10,
            get_value_res.return_value(),
            "total_supply"
        );

        Ok(())
    }

    fn advance_block(n: u8) {
        for _ in 0..n {
            ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        }
    }
}
