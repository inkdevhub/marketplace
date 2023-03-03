import '@polkadot/api-augment';
import type { WeightV2, Weight } from '@polkadot/types/interfaces';
import { expect, use } from 'chai';
import chaiAsPromised from 'chai-as-promised';
import { encodeAddress } from '@polkadot/keyring';
import { FrameSystemAccountInfo } from '@polkadot/types/lookup';
import BN from 'bn.js';
import Market_factory from '../types/constructors/marketplace';
import Market from '../types/contracts/marketplace';
import TestPSP34_factory from '../types/constructors/test_psp34';
import TestPSP34 from '../types/contracts/test_psp34';
import Shiden34_Factory from '../types/constructors/shiden34';
import Shiden34 from '../types/contracts/shiden34';
import Rmrk_Factory from '../types/constructors/rmrk_equippable';
import Rmrk from '../types/contracts/rmrk_equippable';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { ReturnNumber } from '@727-ventures/typechain-types';
import { Hash, NftContractType } from '../types/types-arguments/marketplace';

use(chaiAsPromised);

const MAX_SUPPLY = 888;
const BASE_URI = "ipfs://tokenUriPrefix/";
const COLLECTION_METADATA = "ipfs://collectionMetadata/data.json";
const TOKEN_URI_1 = "ipfs://tokenUriPrefix/1.json";
const TOKEN_URI_5 = "ipfs://tokenUriPrefix/5.json";
const ONE = new BN(10).pow(new BN(18));
const PRICE_PER_MINT = ONE;
const MAX_REF_TIME = '500000000000';
const MAX_PROOF_SIZE = '5242880';

// Create a new instance of contract
const wsProvider = new WsProvider('ws://127.0.0.1:9944');
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' });

describe('Marketplace tests', () => {
  let marketplaceFactory: Market_factory;
  let psp34Factory: TestPSP34_factory;
  let shiden34Factory: Shiden34_Factory;
  let rmrkFactory: Rmrk_Factory;
  let api: ApiPromise;
  let deployer: KeyringPair;
  let bob: KeyringPair;
  let charlie: KeyringPair;
  let marketplace: Market;
  let psp34: TestPSP34;
  let shiden34: Shiden34;
  let rmrk: Rmrk;

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
    shiden34Factory = new Shiden34_Factory(api, deployer);
    rmrkFactory = new Rmrk_Factory(api, deployer);
    marketplace = new Market((await marketplaceFactory.new(deployer.address)).address, deployer, api);
    psp34 = new TestPSP34((await psp34Factory.new()).address, deployer, api);
    shiden34 = new Shiden34((await shiden34Factory.new(
      'default'.split(''),
      'DFT'.split(''),
      'uri'.split(''),
      1000,
      1
    )).address, deployer, api);
    rmrk = new Rmrk((await rmrkFactory.new(
      'default'.split(''),
      'DFT'.split(''),
      'uri'.split(''),
      1000,
      1,
      'meta'.split(''),
      deployer.address,
      1
    )).address, deployer, api);
  }

  it('setup and mint works', async () => {
    await setup();
    const { gasRequired } = await psp34.withSigner(bob).query.mint(bob.address, {u64: 1});
    let mintResult = await psp34.withSigner(bob).tx.mint(bob.address, {u64: 1}, {gasLimit: getEstimatedGas(gasRequired) });
    expect((await psp34.query.totalSupply()).value.unwrap().toNumber()).to.equal(2);
    expect((await psp34.query.balanceOf(bob.address)).value.unwrap()).to.equal(1);
    expect((await psp34.query.ownerOf({ u64: 1 })).value.unwrap()).to.equal(bob.address);
  })

  it('setMarketplaceFee works', async () => {
    await setup();
    let { gasRequired } = await marketplace.query.setMarketplaceFee(120);

    let result = await marketplace.tx.setMarketplaceFee(120, { gasLimit: getEstimatedGas(gasRequired) });
    expect((await marketplace.query.getMarketplaceFee()).value.unwrap()).to.equal(120);
  })

  it('register contract works for the Marketplace owner', async () => {
    await setup();
    await registerContract(deployer);

    const contract = await (await marketplace.query.getRegisteredCollection(psp34.address)).value.unwrap();

    expect(contract.royaltyReceiver).to.be.equal(deployer.address);
    expect(contract.royalty).to.be.equal(100);
    expect(contract.marketplaceIpfs).to.be.equal(toHex(string2ascii('ipfs')));
  });

  it('register contract fails if fee is too high', async () => {
    await setup();

    const ipfs = string2ascii('ipfs');
    const { gasRequired } = await marketplace.withSigner(deployer).query.register(psp34.address, deployer.address, 10001, ipfs);
    const registerResult = await marketplace.withSigner(deployer).query.register(psp34.address, deployer.address, 10001, ipfs, { gasLimit: getEstimatedGas(gasRequired) });

    expect(registerResult.value.unwrap().err.hasOwnProperty('feeTooHigh')).to.be.true;
  });

  it('list / unlist works', async () => {
    await setup();
    await mintToken(bob);
    await registerContract(deployer);
    
    // List token to the marketplace.
    await listToken(bob);
    
    // Check if the token is actually listed.
    expect((await marketplace.query.getPrice(psp34.address, {u64: 1})).value.unwrap()).to.equal(100);

    // Unlist token from the marketplace.
    const { gasRequired } = await marketplace.withSigner(bob).query.unlist(psp34.address, {u64: 1});
    const unlistResult = await marketplace.withSigner(bob).tx.unlist(psp34.address, {u64: 1}, { gasLimit: getEstimatedGas(gasRequired) });
    expect(unlistResult.result?.isFinalized).to.be.true;
    checkIfEventIsEmitted(unlistResult, 'TokenListed', { contract: psp34.address, id: {u64: 1}, price: null });
    
    // Check if the token is actually unlisted.
    const price = await marketplace.query.getPrice(psp34.address, {u64: 1});
    expect(price.value.ok).to.equal(null);
  });

  it('list fails if not a nft owner', async () => {
    await setup();
    await mintToken(bob);
    await registerContract(deployer);
    
    // Try to list token to the marketplace.
    const { gasRequired } = await marketplace.withSigner(charlie).query.list(psp34.address, {u64: 1}, 100);
    const listResult = await marketplace.withSigner(charlie).query.list(psp34.address, {u64: 1}, 100, { gasLimit: getEstimatedGas(gasRequired) });

    expect(listResult.value.unwrap().err.hasOwnProperty('notOwner')).to.be.true;
  });

  it('list fails if token is already listed', async () => {
    await setup();
    await mintToken(bob);
    await registerContract(deployer);
    
    // List token to the marketplace.
    const { gasRequired } = await marketplace.withSigner(bob).query.list(psp34.address, {u64: 1}, 100);
    await marketplace.withSigner(bob).tx.list(psp34.address, {u64: 1}, 100, { gasLimit: getEstimatedGas(gasRequired) });

    // Try to list the same token again.
    const listResult = await marketplace.withSigner(bob).query.list(psp34.address, {u64: 1}, 100, { gasLimit: getEstimatedGas(gasRequired) });

    expect(listResult.value.unwrap().err.hasOwnProperty('itemAlreadyListedForSale')).to.be.true;
  });

  it('unlist fails if token is not listed', async () => {
    await setup();
    await mintToken(bob);
    await registerContract(deployer);
    
    // unlist token to the marketplace.
    const { gasRequired } = await marketplace.withSigner(bob).query.unlist(psp34.address, {u64: 1});
    const unlistResult = await marketplace.withSigner(bob).query.unlist(psp34.address, {u64: 1}, { gasLimit: getEstimatedGas(gasRequired) });

    expect(unlistResult.value.unwrap().err.hasOwnProperty('itemNotListedForSale')).to.be.true;
  });

  it('buy works', async () => {
    await setup();
    await mintToken(charlie);
    await registerContract(deployer);
    await listToken(charlie);

    // Charlie approves marketplace to be operator of the token
    const approveGas = (await psp34.withSigner(charlie).query.approve(marketplace.address, { u64: 1 }, true)).gasRequired;
    let approveResult = await psp34.withSigner(charlie).tx.approve(marketplace.address, { u64: 1 }, true, { gasLimit: getEstimatedGas(approveGas) });

    const deployerOriginalBalance = await getBalance(deployer);
    const bobOriginalBalance = await getBalance(bob);
    const charlieOriginalBalance = await getBalance(charlie);

    // Buy token
    const { gasRequired } = await marketplace.withSigner(bob).query.buy(psp34.address, {u64: 1});
    const buyResult = await marketplace.withSigner(bob).tx.buy(
      psp34.address, 
      {u64: 1},
      { gasLimit: getEstimatedGas(gasRequired), value: new BN('100000000000000000000') });

    expect(buyResult.result?.isFinalized).to.be.true;
    checkIfEventIsEmitted(buyResult, 'TokenBought', { contract: psp34.address, id: {u64: 1}, price: BigInt('100000000000000000000') })

    // Balances check.
    const deployerBalance = await getBalance(deployer);
    const bobBalance = await getBalance(bob);
    const charlieBalance = await getBalance(charlie);
    
    // Check the marketplace fee receiver balance. ATM all royalties go to deployer.
    expect(deployerBalance.eq(deployerOriginalBalance.add(new BN('2000000000000000000')))).to.be.true;
    // Check seller's balance. Should be increased by price - fees
    expect(charlieBalance.toString()).to.be.equal(charlieOriginalBalance.add(new BN('98000000000000000000')).toString());
    // Check the token owner.
    expect((await psp34.query.ownerOf({ u64: 1 })).value.unwrap()).to.equal(bob.address);
    // Check if allowance is unset.
    expect((await psp34.query.allowance(charlie.address, marketplace.address, { u64: 1 })).value.ok).to.equal(false);

    // Try to buy the same token again
    const reBuyResult = await marketplace.withSigner(bob).query.buy(
      psp34.address, 
      {u64: 1},
      { gasLimit: getEstimatedGas(gasRequired), value: new BN('100000000000000000000') });
    expect(reBuyResult.value.unwrap().err.hasOwnProperty('alreadyOwner')).to.be.true;
  });

  it('setContractMetadata works', async () => {
    await setup();
    await registerContract(deployer);
    const marketplace_ipfs = string2ascii('ipfs://test');

    const gas = (await marketplace.withSigner(deployer).query.setContractMetadata(psp34.address, marketplace_ipfs)).gasRequired;
    const approveResult = await marketplace.withSigner(deployer).tx.setContractMetadata(psp34.address, marketplace_ipfs, { gasLimit: getEstimatedGas(gas) });

    const contract = await marketplace.query.getRegisteredCollection(psp34.address);
    expect(contract.value.unwrap().marketplaceIpfs).to.be.equal(toHex(marketplace_ipfs));
  });

  it('setContractMetadata returns error if no contract', async () => {
    await setup();
    const marketplace_ipfs = 'ipfs://test';

    const gas = (await marketplace.withSigner(deployer).query.setContractMetadata(psp34.address, marketplace_ipfs.split(''))).gasRequired;
    const approveResult = await marketplace.withSigner(deployer).query.setContractMetadata(psp34.address, marketplace_ipfs.split(''), { gasLimit: getEstimatedGas(gas) });

    expect(approveResult.value.unwrap().err.hasOwnProperty('notRegisteredContract')).to.be.true;
  });

  it('setNftContractHash works', async () => {
    await setup();
    await registerContract(deployer);
    const hash = string2ascii('h'.repeat(32));

    const gas = (await marketplace.withSigner(deployer).query.setNftContractHash(NftContractType.psp34, hash)).gasRequired;
    await marketplace.withSigner(deployer).tx.setNftContractHash(NftContractType.psp34, hash, {gasLimit: getEstimatedGas(gas)});

    const hashValue = await marketplace.query.nftContractHash(NftContractType.psp34);
    expect(hashValue.value.unwrap()).to.be.equal(toHex(hash));
  });

  it('setNftContractHash fails if not an owner', async () => {
    await setup();
    await registerContract(deployer);
    const hash = string2ascii('h'.repeat(32));

    const gas = (await marketplace.withSigner(bob).query.setNftContractHash(NftContractType.rmrk, hash)).gasRequired;
    const result = await marketplace.withSigner(bob).query.setNftContractHash(NftContractType.rmrk, hash, {gasLimit: getEstimatedGas(gas)});

    expect(result.value.unwrap().err.ownableError).to.equal('CallerIsNotOwner');
  });

  it('PSP34 factory works', async () => {
    const shiden34Hash: Hash = shiden34.abi.info.source.wasmHash.toHex();
    await setup();
    await callFactory(NftContractType.psp34, shiden34Hash);
  });

  it('RMRK factory works', async () => {
    const rmrkHash: Hash = rmrk.abi.info.source.wasmHash.toHex();
    await setup();
    await callFactory(NftContractType.rmrk, rmrkHash);
  });

  async function callFactory(contractType: NftContractType, contractHash: Hash) {
    const marketplace_ipfs = 'ipfs://test';
    const hashGas = (await marketplace.withSigner(deployer).query.setNftContractHash(contractType, contractHash)).gasRequired;
    await marketplace.withSigner(deployer).tx.setNftContractHash(contractType, contractHash, { gasLimit: getEstimatedGas(hashGas) });
    
    const gas = (await marketplace.withSigner(deployer).query.factory(
      string2ascii(marketplace_ipfs),
      bob.address,
      200,
      string2ascii('testNft'),
      string2ascii('TST'),
      string2ascii('nftUri'),
      1000,
      100,
      contractType
    )).gasRequired;
    const factoryResult = await marketplace.withSigner(deployer).tx.factory(
      string2ascii(marketplace_ipfs),
      bob.address,
      200,
      string2ascii('testNft'),
      string2ascii('TST'),
      string2ascii('nftUri'),
      1000,
      100,
      contractType,
      {gasLimit: getEstimatedGas(gas)});
    
    // Check if Shiden34 contract has been deployed
    const instantiatedEvent = factoryResult.result.events.find(x => x.event.method === 'Instantiated' && x.event.section === 'contracts');
    expect(instantiatedEvent).is.not.empty;
    
    const shiden34Address = instantiatedEvent.event.data['contract'].toHuman();
    expect(shiden34Address).is.not.empty;
    checkIfEventIsEmitted(factoryResult, 'CollectionRegistered', { contract: shiden34Address });

    // Check if deployed contract has been registered
    const registerCheckResult = await (await marketplace.query.getRegisteredCollection(shiden34Address)).value.unwrap();
    expect(registerCheckResult).is.not.empty;
    expect(registerCheckResult.royalty).to.be.equal(200);
    expect(registerCheckResult.royaltyReceiver).to.be.equal(bob.address);
    expect(registerCheckResult.marketplaceIpfs).to.be.equal(toHex(string2ascii(marketplace_ipfs)));
  }

  // Helper function to mint a token.
  async function mintToken(signer: KeyringPair): Promise<void> {
    const { gasRequired } = await psp34.withSigner(signer).query.mint(signer.address, {u64: 1});
    const mintResult = await psp34.withSigner(signer).tx.mint(signer.address, {u64: 1}, { gasLimit: getEstimatedGas(gasRequired) });
    expect(mintResult.result?.isFinalized).to.be.true;
    expect((await psp34.query.ownerOf({ u64: 1 })).value.unwrap()).to.equal(signer.address);
  }

  // Helper function to register contract.
  async function registerContract(signer:KeyringPair) {
    const ipfs = string2ascii('ipfs');
    const { gasRequired } = await marketplace.withSigner(signer).query.register(psp34.address, signer.address, 100, ipfs);
    const registerResult = await marketplace.withSigner(signer).tx.register(psp34.address, signer.address, 100, ipfs, { gasLimit: getEstimatedGas(gasRequired) });
    expect(registerResult.result?.isFinalized).to.be.true;
    checkIfEventIsEmitted(registerResult, 'CollectionRegistered', { contract: psp34.address });
  }

  // Helper function to list token for sale.
  async function listToken(signer:KeyringPair) {
    const { gasRequired } = await marketplace.withSigner(signer).query.list(psp34.address, {u64: 1}, 100);
    const listResult = await marketplace.withSigner(signer).tx.list(psp34.address, {u64: 1}, 100, { gasLimit: getEstimatedGas(gasRequired) });
    expect(listResult.result?.isFinalized).to.be.true;
    checkIfEventIsEmitted(listResult, 'TokenListed', { contract: psp34.address, id: {u64: 1}, price: 100 });
  }

  // Helper function to get account balance
  async function getBalance(account:KeyringPair) {
    const balances = await api.query.system.account<FrameSystemAccountInfo>(account.address);

    return balances.data.free;
  }

  function getEstimatedGas(gasRequired: Weight): WeightV2 {
    // For some reason Typechain returns wrong type Weigh, although under the hood
    // WeightV2 structure is stored
    const gasRequiredV2 = gasRequired as unknown as WeightV2;
    return api.registry.createType(
      'WeightV2',
      {
        refTime: gasRequiredV2.refTime.toBn().muln(2),
        proofSize: gasRequiredV2.proofSize.toBn().muln(2),
      }
    ) as WeightV2;
  }
})

// Helper function to parse Events
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function checkIfEventIsEmitted(result: { events?: any }, name: string, args: any): void {
  const event = result.events.find(
    (event: { name: string }) => event.name === name,
  );
  for (const key of Object.keys(event.args)) {
    if (event.args[key] instanceof ReturnNumber) {
      event.args[key] = BigInt(event.args[key]);
    }
  }
  expect(event).eql({ name, args, });
}

// Helper function to get ASCII array from string.
function string2ascii(inputString: string): number[] {
  let result: number[] = [];
  for (var i = 0; i < inputString.length; i ++) {
    result.push(inputString[i].charCodeAt(0));
  }

  return result;
}

// Helper function to get hex string from ASCII array.
function toHex(ascii: number[]): string {
  return '0x' + Buffer.from(ascii).toString('hex');
}