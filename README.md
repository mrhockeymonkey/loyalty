# Loyalty Card 

This project is aimed at creating a free, open source loyalty scheme for small businesses.

It aims to be "as secure" as the traditional paper stamp cards and does not require an app or an account

At this stage this is purely for learning purposes and fun with a single business in mind initially to trial it.

## How it should work

Collecting stamps

- A QR code is displayed to customers
- Scanning the QA code takes the customer to the service...
  - Running on Azure
  - Running on Local RaspberryPi?
- The user is identified by:
  - Phone number
  - Google account (google wallet integration)
  - Apple account (apple wallet integration) (Requires developer license, £100/yr)
- A stamp is added to their card

Redeeming stamps

- User scans QR code and completes stamp card OR user browses to website to view previously completed card
- Option 1: 
  - User or shop clicks "redeem" button
  - The card is cleared of stamps
  - The shop honours the reward
- Option 2: 
  - User clicks redeem
  - QR code is displayed
  - Shop scans this code
  - The card is cleared and a message to shop displayed
  - the shop honours the reward

## Drawbacks

**Without an app or account users might not know how to get to their card...**

By saving a cookie to link them auotmatically to the correct card we can make it easy to get to.
If that cookie is not present they can re-enter their phone number again.

**Phone numbers are public information...**

Its not designed to be secure so this relies on trusting users not to steal friends cards much like paper cards rely on the same thing. 
As a possible improvement a text message confirmation could be used to prevent this from happening.

Using google or apple wallet would also mitigate this issue.

**Phone numbers are personal information...**

We should be clear that we will never share or use this information. 
To not have to deal with GDPR we could make sure the phone number never leave the phone and instead send a hash for identification

(This needs checking)

