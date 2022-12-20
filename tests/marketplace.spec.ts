import '@polkadot/api-augment';
import { expect, use } from 'chai';
import chaiAsPromised from 'chai-as-promised';
import { encodeAddress } from '@polkadot/keyring';
import { FrameSystemAccountInfo } from '@polkadot/types/lookup';
import BN from 'bn.js';
import Market_factory from '../types/constructors/marketplace';
import Market from '../types/contracts/marketplace';
import TestPSP34_factory from '../types/constructors/test_psp34';
import TestPSP34 from '../types/contracts/test_psp34';

import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
// import { AccountId } from '../types/types-arguments/marketplace_contract';
import { ReturnNumber } from '@supercolony/typechain-types';

use(chaiAsPromised);

const MAX_SUPPLY = 888;
const BASE_URI = "ipfs://tokenUriPrefix/";
const COLLECTION_METADATA = "ipfs://collectionMetadata/data.json";
const TOKEN_URI_1 = "ipfs://tokenUriPrefix/1.json";
const TOKEN_URI_5 = "ipfs://tokenUriPrefix/5.json";
const ONE = new BN(10).pow(new BN(18));
const PRICE_PER_MINT = ONE;

// Create a new instance of contract
const wsProvider = new WsProvider('ws://127.0.0.1:9944');
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });

describe('Marketplace tests', () => {
  let marketplaceFactory: Market_factory;
  let psp34Factory: TestPSP34_factory;
  let api: ApiPromise;
  let deployer: KeyringPair;
  let bob: KeyringPair;
  let charlie: KeyringPair;
  let marketplace: Market;
  let psp34: TestPSP34;

  const gasLimit = 18750000000;
  const ZERO_ADDRESS = encodeAddress(
    '0x0000000000000000000000000000000000000000000000000000000000000000',
  );
  let gasRequired: bigint;

  async function setup(): Promise<void> {
    api = await ApiPromise.create({ provider: wsProvider });
    deployer = keyring.addFromUri('//Alice');
    bob = keyring.addFromUri('//Bob');
    charlie = keyring.addFromUri('//Charlie');
    marketplaceFactory = new Market_factory(api, deployer);
    psp34Factory = new TestPSP34_factory(api, deployer);
    marketplace = new Market((await marketplaceFactory.new(deployer.address)).address, deployer, api);
    psp34 = new TestPSP34((await psp34Factory.new()).address, deployer, api);
  }

  it('setup and mint works', async () => {
    await setup();
    const { gasRequired } = await psp34.withSigner(bob).query.mint(bob.address, {u64: 1});
    let mintResult = await psp34.withSigner(bob).tx.mint(bob.address, {u64: 1}, {gasLimit: gasRequired * 2n });
    expect((await psp34.query.totalSupply()).value.rawNumber.toNumber()).to.equal(2);
    expect((await psp34.query.balanceOf(bob.address)).value).to.equal(1);
    expect((await psp34.query.ownerOf({ u64: 1 })).value).to.equal(bob.address);
  })

  it('setMarketplaceFee works', async () => {
    await setup();
    let { gasRequired } = await marketplace.query.setMarketplaceFee(120);

    let result = await marketplace.tx.setMarketplaceFee(120, { gasLimit: gasRequired });
    expect((await marketplace.query.getMarketplaceFee()).value).to.equal(120);
  })

  it('list / unlist works', async () => {
    await setup();
    await mintToken(bob);
    await registerContract(bob);
    
    // List token to the marketplace.
    await listToken(bob);
    
    // Check if the token is actually listed.
    expect((await marketplace.query.getPrice(psp34.address, {u64: 1})).value).to.equal(100);

    // Unlist token from the marketplace.
    const { gasRequired } = await marketplace.withSigner(bob).query.unlist(psp34.address, {u64: 1});
    const unlistResult = await marketplace.withSigner(bob).tx.unlist(psp34.address, {u64: 1}, { gasLimit: gasRequired * 2n });
    expect(unlistResult.result?.isInBlock).to.be.true;
    
    // Check if the token is actually unlisted.
    expect((await marketplace.query.getPrice(psp34.address, {u64: 1})).value).to.equal(null);
  });

  // it('list fails if not a nft owner', async () => {
  //   await setup();
  //   await mintToken(bob);
  //   await registerContract(bob);
    
  //   // Try to list token to the marketplace.
  //   const { gasRequired } = await marketplace.withSigner(charlie).query.list(psp34.address, {u64: 1}, 100);
  //   const listResult = await marketplace.withSigner(charlie).tx.list(psp34.address, {u64: 1}, 100, { gasLimit: gasRequired * 2n });
  //   // TODO how to handle errors returned by a contract
  //   console.log('LR', listResult);
  // });

  it('buy works', async () => {
    await setup();
    await mintToken(charlie);
    await registerContract(bob);
    await listToken(charlie);

    // Charlie approves marketplace to be operator of the token
    const approveGas = (await psp34.withSigner(charlie).query.approve(marketplace.address, { u64: 1 }, true)).gasRequired;
    let approveResult = await psp34.withSigner(charlie).tx.approve(marketplace.address, { u64: 1 }, true, { gasLimit: approveGas });

    const deployerOriginalBalance = await getBalance(deployer);
    const bobOriginalBalance = await getBalance(bob);
    const charlieOriginalBalance = await getBalance(charlie);

    console.log(deployerOriginalBalance.toHuman(), bobOriginalBalance.toHuman(), charlieOriginalBalance.toHuman());

    // Buy token
    const { gasRequired } = await marketplace.withSigner(deployer).query.buy(psp34.address, {u64: 1});
    const buyResult = await marketplace.withSigner(deployer).tx.buy(
      psp34.address, 
      {u64: 1},
      { gasLimit: gasRequired * 2n, value: new BN('100000000000000000000') });

    expect(buyResult.result?.isInBlock).to.be.true;

    // Balances check.
    const deployerBalance = await getBalance(deployer);
    const bobBalance = await getBalance(bob);
    const charlieBalance = await getBalance(charlie);
    console.log(deployerBalance.toHuman(), bobBalance.toHuman(), charlieBalance.toHuman());
    
    // Check the marketplace fee receiver balance
    // TODO needed to convert BN to string since BN comparison doesnt't work.
    expect(bobBalance.toString()).to.be.equal(bobOriginalBalance.add(new BN('1000000000000000000')).toString());
    // Check seller's balance
    expect(charlieBalance.toString()).to.be.equal(charlieOriginalBalance.add(new BN('98000000000000000000')).toString());
    // Check a new token owner
    expect((await psp34.query.ownerOf({ u64: 1 })).value).to.equal(deployer.address);
    // Check if allowance is unset.
    expect((await psp34.query.allowance(charlie.address, marketplace.address, { u64: 1 })).value).to.equal(false);
  });

  it('setContractMetadata works', async () => {
    await setup();
    await registerContract(bob);
    const metadata = 'ipfs://test';

    const gas = (await marketplace.withSigner(deployer).query.setContractMetadata(psp34.address, metadata)).gasRequired;
    const approveResult = await marketplace.withSigner(deployer).tx.setContractMetadata(psp34.address, metadata, { gasLimit: gas });

    const contract = await (await marketplace.query.getContract(psp34.address)).value;
    expect(contract.metadata).to.be.equal(metadata);
  });

  it('setContractMetadata returns error if no contract', async () => {
    await setup();
    const metadata = 'ipfs://test';

    const gas = (await marketplace.withSigner(deployer).query.setContractMetadata(psp34.address, metadata)).gasRequired;
    const approveResult = await marketplace.withSigner(deployer).query.setContractMetadata(psp34.address, metadata, { gasLimit: gas });

    console.log(approveResult);
    // expect(hex2a(mintResult.value.err.custom)).to.be.equal('BadMintValue');
  });

  // Helper function to mint a token.
  async function mintToken(signer: KeyringPair): Promise<void> {
    const { gasRequired } = await psp34.withSigner(signer).query.mint(signer.address, {u64: 1});
    const mintResult = await psp34.withSigner(signer).tx.mint(signer.address, {u64: 1}, { gasLimit: gasRequired * 2n });
    expect(mintResult.result?.isInBlock).to.be.true;
    expect((await psp34.query.ownerOf({ u64: 1 })).value).to.equal(signer.address);
  }

  // Helper function to register contract.
  async function registerContract(signer:KeyringPair) {
    const { gasRequired } = await marketplace.withSigner(bob).query.register(psp34.address, bob.address, 100);
    const registerResult = await marketplace.withSigner(bob).tx.register(psp34.address, bob.address, 100, { gasLimit: gasRequired * 2n });
    expect(registerResult.result?.isInBlock).to.be.true;
    // expect((await marketplace.query.getPrice(psp34.address, {u64: 1})).value).to.equal(100);
  }

  // Helper function to list token for sale.
  async function listToken(signer:KeyringPair) {
    const { gasRequired } = await marketplace.withSigner(signer).query.list(psp34.address, {u64: 1}, 100);
    const listResult = await marketplace.withSigner(signer).tx.list(psp34.address, {u64: 1}, 100, { gasLimit: gasRequired * 2n });
    expect(listResult.result?.isInBlock).to.be.true;
  }

  // Helper function to get account balance
  async function getBalance(account:KeyringPair) {
    const balances = await api.query.system.account<FrameSystemAccountInfo>(account.address);

    return balances.data.free;
  }
})

// Helper function to parse Events
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function emit(result: { events?: any }, name: string, args: any): void {
  const event = result.events.find(
    (event: { name: string }) => event.name === name,
  );
  for (const key of Object.keys(event.args)) {
    if (event.args[key] instanceof ReturnNumber) {
      event.args[key] = event.args[key].toNumber();
    }
  }
  expect(event).eql({ name, args, });
}

// Helper function to convert error code to string
function hex2a(error: any): string {
  var hex = error.toString(); //force conversion
  var str = '';
  for (var i = 0; i < hex.length; i += 2)
    str += String.fromCharCode(parseInt(hex.substr(i, 2), 16));
  return str.substring(1);
}