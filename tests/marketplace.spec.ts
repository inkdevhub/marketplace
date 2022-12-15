import { expect, use } from 'chai';
import chaiAsPromised from 'chai-as-promised';
import { encodeAddress } from '@polkadot/keyring';
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
    console.log(result);
    expect((await marketplace.query.getMarketplaceFee()).value).to.equal(120);
  })

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