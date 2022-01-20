const nearAPI = require('near-api-js');
const { KeyPair, Account, utils: { format: { parseNearAmount }} } = nearAPI;

beforeAll(async function () {
  // NOTE: nearlib and nearConfig are made available by near-cli/test_environment
  const near = await nearlib.connect(nearConfig)
  window.accountId = nearConfig.contractName
  window.contract = await near.loadContract(nearConfig.contractName, {
    viewMethods: ['get_credits'],
    changeMethods: ['new', 'deposit', 'play'],
    sender: window.accountId
  })

  window.walletConnection = {
    requestSignIn() {
    },
    signOut() {
    },
    isSignedIn() {
      return true
    },
    getAccountId() {
      return window.accountId
    }
  }

  console.log('Initializing contract...');
  await initContract();
  console.log('Contract initialized.');
})

async function initContract() {
  try {
    await window.contract.new({ owner_id: window.accountId });
  } catch (e) {
    if (!/Already initialized!/.test(e.toString())) {
      throw e;
    }
  }
}

async function getContract(account) {
  return new Contract(account || contractAccount, contractName, {
    ...contractMethods,
    signer: account || undefined,
  });
}

const GAS = "200000000000000";

test('get_credits', async () => {
  const credits = await window.contract.get_credits({ account_id: window.accountId })
  expect(credits).toEqual('0');
});

test('deposit', async () => {
  await window.contract.deposit({}, GAS, parseNearAmount('10'));
  const credits = await window.contract.get_credits({ account_id: window.accountId })
  expect(credits).toEqual('10000000000000000000000000');
});
