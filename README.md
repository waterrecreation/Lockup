# Lockup
This contract is a lockup facility template. Creators who has tokens should deploy this contract from [Facility-Factory]. Creators make claim lists into this contract, and specify a lockup period so users in lists can get tokens periodicly.

Contents

* [Terminology](#terminology)
* [Function specification](#function-specification)

## Terminology

* `owner_id`: The owner of this contract, which is creator that determined by [Facility-Factory].
* `tokens`: Registered tokens, contains tasks and balance for every token.

## Function specification

### Contract initialization

Initial function is called by [Facility-Factory] when this contract is deployed.

### Lockup task operation

Creator should specify token, claim list, start time, close time, vesting period and amount. Time related arguments should be in nanosecond timestamp.

### Claim operation

Claim acion checks if a user is in a given task, and check if there's any amount of token to be claimed. Then after claim, the task records the timestamp that user claims. 


  [Facility-Factory]: https://github.com/27s-io/Facility-Factory
