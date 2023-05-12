#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
pub mod contract {
    use ink::env::call::Selector;
    use ink::prelude::vec::Vec;
    use scheduler_extension::*;

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
        pub fn schedule(
            &mut self,
            when: BlockNumber,
            id: u32,
        ) -> Result<(), SchedulerError> {
            let mut data = Vec::new();
            let mut selector: Vec<u8> = Selector::new(ink::selector_bytes!("increase_value"))
                .to_bytes()
                .to_vec();
            data.append(&mut selector);

            let call_input = ContractCallInput {
                dest: self.env().account_id(),
                data,
                gas_limit: (3951114240u64, 125952u64),
                storage_deposit_limit: None,
                value: 0,
                max_weight: 1_000_000_000_000u64,
            };

            SchedulerExtension::schedule(when, id, call_input)
        }

        #[ink(message)]
        pub fn cancel(&mut self, id: u32) -> Result<(), SchedulerError> {
            SchedulerExtension::cancel(id)
        }

        #[ink(message)]
        pub fn increase_value(&mut self) {
            self.value += 10;
        }

        #[ink(message)]
        pub fn get_value(&self) -> u32 {
            self.value
        }

        #[ink(message)]
        pub fn get_block_number(&self) -> u32 {
            self.env().block_number()
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use crate::contract::SchedulerRef;
    use ink_e2e::build_message;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;
    use crate::{contract_call, contract_query, advance_one_block};
    use subxt::dynamic::Value;

    #[ink_e2e::test]
    async fn schedule_set_value_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // given
        let constructor = SchedulerRef::new();
        let contract_acc_id = client
            .instantiate("scheduler_example", &ink_e2e::alice(), constructor, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        let block_number = contract_query!(
            client,
            SchedulerRef,
            &ink_e2e::alice(),
            contract_acc_id,
            |s| s.get_block_number()
        );

        // when
        contract_call!(
            client,
            SchedulerRef,
            &ink_e2e::alice(),
            contract_acc_id,
            |s| s.schedule(block_number + 2, 1)
        );

        // it is possible to advance block with `set_block_timestamp`
        // but it is only on the contract env, so it will no trigger calls on pallet-schedule
        // that use on_initialize hook. That is why here we send two tx that will advance 2 blocks
        advance_one_block!(client);
        advance_one_block!(client);
        
        client
            .runtime_call(
                &ink_e2e::alice(),
                "System",
                "remark",
                vec![Value::from_bytes("0x0101".as_bytes())],
            )
            .await
            .expect("system remark call failed");
        client
            .runtime_call(
                &ink_e2e::alice(),
                "System",
                "remark",
                vec![Value::from_bytes("0x0101".as_bytes())],
            )
            .await
            .expect("system remark call failed");

        // then
        let value = contract_query!(
            client,
            SchedulerRef,
            &ink_e2e::alice(),
            contract_acc_id,
            |s| s.get_value()
        );

        assert_eq!(10, value);

        Ok(())
    }

    #[ink_e2e::test]
    async fn cancel_call_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // given
        let constructor = SchedulerRef::new();
        let contract_acc_id = client
            .instantiate("scheduler_example", &ink_e2e::alice(), constructor, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;
        let block_number = 1000;

        // when
        contract_call!(
            client,
            SchedulerRef,
            &ink_e2e::alice(),
            contract_acc_id,
            |s| s.schedule(block_number + 2, 1)
        );
        contract_call!(
            client,
            SchedulerRef,
            &ink_e2e::alice(),
            contract_acc_id,
            |s| s.cancel(1)
        );

        advance_one_block!(client);
        advance_one_block!(client);

        // then
        let value = contract_query!(
            client,
            SchedulerRef,
            &ink_e2e::alice(),
            contract_acc_id,
            |s| s.get_value()
        );

        assert_eq!(0, value);

        Ok(())
    }
}

#[macro_export]
macro_rules! contract_call {
    ($client:expr, $contract_ref:ident, $signer:expr, $contract_account:expr, $closure:expr) => {
        $client
            .call(
                $signer,
                build_message::<$contract_ref>($contract_account.clone()).call($closure),
                0,
                None,
            )
            .await
            .expect("call failed");
    };
}

#[macro_export]
macro_rules! contract_query {
    ($client:expr, $contract_ref:ident, $signer:expr, $contract_account:expr, $closure:expr) => {
        $client
            .call_dry_run(
                $signer,
                &build_message::<$contract_ref>($contract_account.clone()).call($closure),
                0,
                None,
            )
            .await
            .return_value()
    };
}

// it is possible to advance block with `set_block_timestamp`
// but it is only on the contract env, so it will no trigger calls on pallet-schedule
// that use on_initialize hook. That is why here we send two tx that will advance 2 blocks
#[macro_export]
macro_rules! advance_one_block {
    ($client:expr) => {
        $client
            .runtime_call(
                &ink_e2e::alice(),
                "System",
                "remark",
                vec![Value::from_bytes("0x0101".as_bytes())],
            )
            .await
            .expect("system remark call failed")
    };
}
