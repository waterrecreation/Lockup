<template>
  <div>
    <button class="link" style="float: right" v-on:click="logout">Sign out</button>
    <main> 
      <form v-on:submit.prevent="addToken">
        <fieldset ref="addtoken">
          <label
            style="display:block; color:var(--gray);margin-bottom:0.5em;"
          >Select Token</label>
          <select v-model="token" @change="changeSelect">
            <option v-for="t in token_list" :key="t.symbol" :value="t">{{t.symbol}}</option>
          </select>
          <div style="margin-bottom: 30px;">Balance: {{balance}}</div>
          <label
            style="display:block; color:var(--gray);margin-bottom:0.5em;"
          >Add Token</label>
          <div style="display:flex">
            <input v-model="token_id" autocomplete="off" id="greeting" style="flex:1" />
            <button id="save" style="border-radius:0 5px 5px 0">Save</button>
          </div>
        </fieldset>
      </form>
      <form v-on:submit.prevent="addTask">
        <fieldset ref="addtask">
          <label
            style="display:block; color:var(--gray);margin-bottom:0.7em;"
          >Add Task</label>
          <label
            style="display:block; color:var(--gray);margin-bottom:0.5em;"
          >Amount</label>
          <div style="display:flex;">
            <input v-model="amount" autocomplete="off" id="greeting" style="flex:1" />
          </div>
          <label
            style="display:block; color:var(--gray);margin-bottom:0.5em;"
          >Vesting period</label>
          <div style="display:flex;">
            <input v-model="vesting_period" autocomplete="off" id="greeting" style="flex:1" />
          </div>
          <label
            style="display:block; color:var(--gray);margin-bottom:0.5em;"
          >Start Time</label>
          <div style="display:flex;">
            <input v-model="start_time" autocomplete="off" id="greeting" style="flex:1" />
          </div>
          <label
            style="display:block; color:var(--gray);margin-bottom:0.5em;"
          >End Time</label>
          <div style="display:flex;">
            <input v-model="end_time" autocomplete="off" id="greeting" style="flex:1" />
          </div>
          <label
            style="display:block; color:var(--gray);margin-bottom:0.5em;"
          >Claimers</label>
          <div style="display:flex;">
            <input v-model="claimers" autocomplete="off" id="greeting" style="flex:1" />
          </div>
          <button id="save" style="border-radius:5px; margin-top: 20px">Submit</button>
        </fieldset>
      </form>
      <fieldset ref="claim">
        <label
          style="display:block; color:var(--gray);margin-bottom:0.7em;"
        >Claim</label>
        <div v-for="(item, i) in tasks" :key="i">
          <label
            style="display:block; color:var(--gray);margin-bottom:0.5em;"
          >{{item.should_claim}}</label>
          <button @click="claim_linkdrop(i)" id="save" style="border-radius:5px; margin-top: 20px">Claim</button>
        </div>
      </fieldset>
      <hr />
    </main>

    <Notification
      v-show="notificationVisible"
      ref="notification"
      :networkId="networkId"
      :msg="'called method: set_greeting'"
      :contractId="contractId"
      :visible="false"
    />
  </div>
</template>

<script>
import { logout } from "../utils"

import Notification from "./Notification.vue"
import { getTokenMetadata, getTokenBalance } from '../utils'

export default {
  name: "SignedIn",

  beforeMount() {
    if (this.isSignedIn) {
      this.getTokenList()
    }
  },

  components: {
    Notification,
  },

  data: function () {
    return {
      token_list: [],
      token: {},
      token_id: '',
      amount: 0,
      start_time: 0,
      end_time: 0,
      vesting_period: 0,
      claimers: "",
      balance: 0,
      notificationVisible: false,
      tasks: [],
      index: 0
    }
  },

  computed: {
    isSignedIn() {
      return window.walletConnection? window.walletConnection.isSignedIn(): false
    },
    accountId() {
      return window.accountId
    },
    contractId() {
      return window.contract? window.contract.contractId: null
    },
    networkId() {
      return window.networkId
    },
  },

  methods: {
    getTokenList() {
      //retrieve greeting
      window.contract
        .get_token_list({})
        .then( async res => {
          let token_info = []
          for (let i = 0; i < res.length; i ++) {
            let info = await getTokenMetadata(res[i])
            info.token_id = res[i]
            token_info.push(info)
          }
          console.log(token_info)
          this.token_list = token_info
        })
      
    },

    

    addToken: async function () {
      // fired on form submit button used to update the greeting

      // disable the form while the value gets updated on-chain
      this.$refs.addtoken.disabled = true

      try {
        
        // make an update call to the smart contract
        await window.contract.add_token({
          // pass the new greeting
          token_id: this.token_id,
        }, "300000000000000", "1250000000000000000000")
      } catch (e) {
        alert(
          "Something went wrong! " +
            "Maybe you need to sign out and back in? " +
            "Check your browser console for more info."
        )
        throw e //re-throw
      } finally {
        // re-enable the form, whether the call succeeded or failed
        this.$refs.addtoken.disabled = false
      }

      // update savedGreeting with persisted value

      this.notificationVisible = true //show new notification

      // remove Notification again after css animation completes
      // this allows it to be shown again next time the form is submitted
      setTimeout(() => {
        this.notificationVisible = false
      }, 11000)

    },

    async changeSelect() {
      this.balance = await getTokenBalance(this.token.token_id)
      this.tasks = await window.contract.get_tasks_by_token_id({token_id: this.token.token_id, sender: window.accountId})
      console.log(this.tasks)
    },

    addTask: async function () {
      // fired on form submit button used to update the greeting

      // disable the form while the value gets updated on-chain
      this.$refs.addtask.disabled = true

      let claimers = this.claimers.split(";")

      try {
        await window.contract.add_task({
          account_list: claimers,
          amount: this.amount,
          start_time: this.start_time,
          end_time: this.end_time,
          token_id: this.token.token_id,
          vesting_period: this.vesting_period
        }, "300000000000000", 0)
      } catch (e) {
        alert(
          "Something went wrong! " +
            "Maybe you need to sign out and back in? " +
            "Check your browser console for more info."
        )
        throw e //re-throw
      } finally {
        // re-enable the form, whether the call succeeded or failed
        this.$refs.addtask.disabled = false
      }

      // update savedGreeting with persisted value

      this.notificationVisible = true //show new notification

      // remove Notification again after css animation completes
      // this allows it to be shown again next time the form is submitted
      setTimeout(() => {
        this.notificationVisible = false
      }, 11000)

    },

    claim_linkdrop: async function (index) {
      // fired on form submit button used to update the greeting

      // disable the form while the value gets updated on-chain
      this.$refs.claim.disabled = true
      

      try {
        await window.contract.claim({token_id: this.token.token_id, task_index: index}, "300000000000000", 0)
      } catch (e) {
        alert(
          "Something went wrong! " +
            "Maybe you need to sign out and back in? " +
            "Check your browser console for more info."
        )
        throw e //re-throw
      } finally {
        // re-enable the form, whether the call succeeded or failed
        this.$refs.claim.disabled = false
      }

      // update savedGreeting with persisted value

      this.notificationVisible = true //show new notification

      // remove Notification again after css animation completes
      // this allows it to be shown again next time the form is submitted
      setTimeout(() => {
        this.notificationVisible = false
      }, 11000)

    },

    logout: logout,
  },
}
</script>
