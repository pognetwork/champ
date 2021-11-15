# Importance Farming / Sybil attacks

**Description:** <br>
Farming Importance is using multiple Wallet that generate importance and delegating their importance to one main Wallet. This Wallet will then have a boosted importance and therefore may be able to attack the network. This is called a Sybil attack

In order to improve our consensus, we plan on creating an algorithm that identifies these farming Wallets and possibly bans them from the network.

## Node Evidence

| Normal Wallets                  | Farming Wallets                                                   | Main Wallet                                    |
|---------------------------------|-------------------------------------------------------------------|------------------------------------------------|
| May have high or low money      | Probably medium to low money (most money on main account)         | May have short age but many delegating to them |
| Probably random trx             | Probably very even amount of trx                                  | May trx with only Farming Wallets              |
| May delegate at any age         | Probably delegate with low age                                    |                                                |
| May change delegate often       | Probably never changes delegate                                   |                                                |
| May have trx with random people | Probably only exchanges with other Farming Wallets or Main Wallet |                                                |
| May get money from random person| Probably gets initial money from Main Wallet                      |                                                | 

## Metadata Evidence

In addition, there is metadata we might use to get more evidence:
- How often is logged into the Wallet -> Farming Wallets may be left on their own for longer
- Are the transactions automatic or by hand -> Farming Wallets may have automatic transfers to boost importance
- Wallets all accessed from the same Browser -> Farming Wallets probably managed by the same person using the same Browser