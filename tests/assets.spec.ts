import { expect, use } from 'chai';
import chaiAsPromised from 'chai-as-promised';
import assets_constructor from '../types/constructors/asset_wrapper';
import assets_contract from '../types/contracts/asset_wrapper';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import {buildTx} from "./helper";
import {afterEach} from "mocha";
import {WeightV2} from "@polkadot/types/interfaces";
import {stringToHex} from "@polkadot/util/string/toHex";

use(chaiAsPromised);

// Create a new instance of contract
const wsProvider = new WsProvider('ws://127.0.0.1:9944');
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });

const ASSET_ID = 1

describe('ASSETS', () => {
    let api: ApiPromise;
    let alice: KeyringPair;
    let bob: KeyringPair;
    let charlie: KeyringPair;
    let assetsConstructor: assets_constructor
    let assets: assets_contract;

    let gasRequired: WeightV2;

    beforeEach(async function() {
        api = await ApiPromise.create({ provider: wsProvider });
        alice = keyring.addFromUri('//Alice');
        bob = keyring.addFromUri('//Bob');
        charlie = keyring.addFromUri('//Charlie');
        assetsConstructor = new assets_constructor(api, alice);
        assets = new assets_contract((await assetsConstructor.new()).address, alice, api);
    })

    it('create works', async () => {
        let { gasRequired: gas }  = await assets.query.create(ASSET_ID,1, {
            value: 1000000,
        });
        await assets.tx.create(ASSET_ID, 1,{gasLimit: gas, value: 1000000 });

        const prefix = api.consts.system.ss58Prefix
        // @ts-ignore
        const assetAddress = keyring.encodeAddress(assets.address, prefix);
        // @ts-ignore
        await expect((await api.query.assets.asset(1)).unwrapOrDefault().owner.toString()).to.equal(assetAddress)
    })

    it('mint works', async () => {
        let { gasRequired: gas }  = await assets.query.create(2,1, {
            value: 1000000,
        });
        await assets.tx.create(2, 1,{gasLimit: gas, value: 1000000 });

        let { gasRequired: gas2 }  = await assets.query.mint(2, alice.address,1000);
        await assets.tx.mint(2,  alice.address,1000,{gasLimit: gas2 });

        // @ts-ignore
        await expect((await api.query.assets.account(2, alice.address)).unwrapOrDefault().balance.toNumber()).to.equal(1000)
    })

    it('burn works', async () => {
        let { gasRequired: gas }  = await assets.query.create(3,1, {
            value: 1000000,
        });
        await assets.tx.create(3, 1,{gasLimit: gas, value: 1000000 });

        let { gasRequired: gas2 }  = await assets.query.mint(3, alice.address,1000);
        await assets.tx.mint(3, alice.address, 1000,{gasLimit: gas2 });

        let { gasRequired: gas3 }  = await assets.query.burn(3, alice.address,100);
        await assets.tx.burn(3,  alice.address,100,{gasLimit: gas3 });

        // @ts-ignore
        await expect((await api.query.assets.account(3, alice.address)).unwrapOrDefault().balance.toNumber()).to.equal(1000 - 100)
    })

    it('balance_of and total_supply are correct', async () => {
        let { gasRequired: gas }  = await assets.query.create(4,1, {
            value: 1000000,
        });
        await assets.tx.create(4, 1,{gasLimit: gas, value: 1000000 });

        let { gasRequired: gas2 }  = await assets.query.mint(4, alice.address,1000);
        await assets.tx.mint(4, alice.address,1000,{gasLimit: gas2 });

        // @ts-ignore
        await expect((await assets.query.balanceOf(4, alice.address)).value.unwrap().toNumber()).to.equal(1000)

        // @ts-ignore
        await expect((await assets.query.totalSupply(4)).value.unwrap().toNumber()).to.equal(1000)
    })

    it('approve transfer and check allowance', async () => {
        let { gasRequired: gas }  = await assets.query.create(5,1, {
            value: 1000000,
        });
        await assets.tx.create(5, 1,{gasLimit: gas, value: 1000000 });

        let { gasRequired: gas2 }  = await assets.query.mint(5, assets.address, 1000);
        await assets.tx.mint(5, assets.address,1000,{gasLimit: gas2 });

        let { gasRequired: gas3 }  = await assets.query.approveTransfer(5,bob.address, 100);
        await assets.tx.approveTransfer(5, bob.address, 100, {gasLimit: gas3 });

        // @ts-ignore
        await expect((await assets.query.allowance(5, assets.address, bob.address)).value.unwrap().toNumber()).to.equal(100)
    })

    it('approve transfer, transfer and check balances', async () => {
        let { gasRequired: gas }  = await assets.query.create(6,1, {
            value: 1000000,
        });
        await assets.tx.create(6, 1,{gasLimit: gas, value: 1000000 });

        let { gasRequired: gas2 }  = await assets.query.mint(6, assets.address,1000);
        await assets.tx.mint(6, assets.address, 1000,{gasLimit: gas2 });

        let { gasRequired: gas3 }  = await assets.query.approveTransfer(6, bob.address, 100);
        await assets.tx.approveTransfer(6, bob.address, 100, {gasLimit: gas3 });

        // with Buildtx
        // let { gasRequired: gas4}  = await assets.withSigner(bob).query.transferApproved(6, alice.address, charlie.address, 50);
        // await assets.withSigner(bob).tx.transferApproved(6, alice.address, charlie.address, 50, {gasLimit: gas4});

        // @ts-ignore
        await expect((await assets.query.allowance(6, alice.address, bob.address)).value.unwrap().toNumber()).to.equal(50)

        // @ts-ignore
        await expect((await assets.query.balanceOf(6, alice.address)).value.unwrap().toNumber()).to.equal(950)

        // @ts-ignore
        await expect((await assets.query.balanceOf(6, bob.address)).value.unwrap().toNumber()).to.equal(0)

        // @ts-ignore
        await expect((await assets.query.balanceOf(6, charlie.address)).value.unwrap().toNumber()).to.equal(50)
    })
    //
    // it('cancel approval', async () => {
    //     let { gasRequired: gas }  = await assets.query.create(ASSET_ID,1, {
    //         value: 1000000,
    //     });
    //     await assets.tx.create(ASSET_ID, 1,{gasLimit: gas, value: 1000000 });
    //
    //     let { gasRequired: gas2 }  = await assets.query.mint(ASSET_ID, alice.address,1000);
    //     await assets.tx.mint(ASSET_ID, alice.address,1000,{gasLimit: gas2 });
    //
    //     let { gasRequired: gas3 }  = await assets.query.approveTransfer(ASSET_ID,bob.address, 100);
    //     await assets.tx.approveTransfer(ASSET_ID, bob.address, 100, {gasLimit: gas3 });
    //
    //     let { gasRequired: gas4 }  = await assets.query.cancelApproval(ASSET_ID,bob.address);
    //     await assets.tx.cancelApproval(ASSET_ID, bob.address,{gasLimit: gas4 });
    //
    //     // @ts-ignore
    //     await expect((await assets.withSigner(bob).query.transferApproved(ASSET_ID, alice.address, charlie.address, 100)).value.unwrap().err).to.equal('Unapproved')
    //
    //     // @ts-ignore
    //     await expect((await assets.query.allowance(ASSET_ID, alice.address, bob.address)).value.unwrap().toNumber()).to.equal(0)
    //
    //     // @ts-ignore
    //     await expect((await assets.query.balanceOf(ASSET_ID, alice.address)).value.unwrap().toNumber()).to.equal(1000)
    //
    //     // @ts-ignore
    //     await expect((await assets.query.balanceOf(ASSET_ID, bob.address)).value.unwrap().toNumber()).to.equal(0)
    //
    //     // @ts-ignore
    //     await expect((await assets.query.balanceOf(ASSET_ID, charlie.address)).value.unwrap().toNumber()).to.equal(0)
    // })
    //
    // it('set metadata and checks', async () => {
    //     let { gasRequired: gas }  = await assets.query.create(ASSET_ID,1, {
    //         value: 1000000,
    //     });
    //     await assets.tx.create(ASSET_ID, 1,{gasLimit: gas, value: 1000000 });
    //
    //     let { gasRequired: gas3 }  = await assets.query.setMetadata(ASSET_ID, 'Shiden Token' as unknown as string[], 'TTT' as unknown as string[], 18);
    //     await assets.tx.setMetadata(ASSET_ID, 'Shiden Token' as unknown as string[], 'TTT' as unknown as string[], 18, {gasLimit: gas3 });
    //
    //     // @ts-ignore
    //     await expect((await assets.query.metadataName(1)).value.unwrap()).to.equal(stringToHex('Shiden Token'))
    //
    //     // @ts-ignore
    //     await expect((await assets.query.metadataSymbol(1)).value.unwrap()).to.equal(stringToHex('TTT'))
    //
    //     // @ts-ignore
    //     await expect((await assets.query.metadataDecimals(ASSET_ID, alice.address, bob.address)).value.unwrap()).to.equal(18)
    // })
})
