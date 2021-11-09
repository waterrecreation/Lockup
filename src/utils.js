import { connect, Contract, keyStores, WalletConnection } from 'near-api-js'
import getConfig from './config'

const nearConfig = getConfig('development')

console.log(nearConfig)

// Initialize contract & set global variables
export async function initContract() {
  // Initialize connection to the NEAR testnet
  const near = await connect(Object.assign({ deps: { keyStore: new keyStores.BrowserLocalStorageKeyStore() } }, nearConfig))

  // Initializing Wallet based Account. It can work with NEAR testnet wallet that
  // is hosted at https://wallet.testnet.near.org
  window.walletConnection = new WalletConnection(near)

  // Getting the Account ID. If still unauthorized, it's just empty string
  window.accountId = window.walletConnection.getAccountId()

  // Initializing our contract APIs by contract name and configuration
  window.contract = await new Contract(window.walletConnection.account(), nearConfig.contractName, {
    // View methods are read only. They don't modify the state, but usually return some value.
    viewMethods: ['get_token_list', 'get_tasks', 'get_tasks_by_token_id'],
    // Change methods can modify the state. But you don't receive the returned value when called.
    changeMethods: ['add_token', 'add_task', 'claim'],
  })
}

export async function getTokenMetadata(token_id) {
  let contract = await new Contract(window.walletConnection.account(), token_id, {
    // View methods are read only. They don't modify the state, but usually return some value.
    viewMethods: ['ft_metadata'],
    // Change methods can modify the state. But you don't receive the returned value when called.
    changeMethods: [],
  })
  return contract.ft_metadata()
}

export async function getTokenBalance(token_id) {
  let contract = await new Contract(window.walletConnection.account(), token_id, {
    // View methods are read only. They don't modify the state, but usually return some value.
    viewMethods: ['ft_balance_of'],
    // Change methods can modify the state. But you don't receive the returned value when called.
    changeMethods: [],
  })
  return contract.ft_balance_of({account_id: nearConfig.contractName})
}

export function logout() {
  window.walletConnection.signOut()
  // reload page
  window.location.replace(window.location.origin + window.location.pathname)
}

export function login() {
  // Allow the current app to make calls to the specified contract on the
  // user's behalf.
  // This works by creating a new access key for the user's account and storing
  // the private key in localStorage.
  window.walletConnection.requestSignIn(nearConfig.contractName)
}
