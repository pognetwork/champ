# Security at Pog.Network

## Threat Model
![](threat.drawio.svg)

## Threats
- ### All outgoing connections
    **Attacks:**
    DDOS: We plan to add rate limiting and we will guide Node Operators for optimal setup as we won't run the nodes ourselves.
    (_Denial of Service, High Severity_)
    Insecure SSL version: In release we plan to refuse connections with a TLS version different than 1.3 and refuse insecure connections. Currently, everything is accepted for development purposes.
    (_Tamperling, High Severity_)
    Unencrypted origin traffic:
    
- ### Web Clients
    **Attacks:**
    XSS Attacks: `Mitigated` because React sanitizes all user input and therefore prevents accidental script injections. Furthermore, we plan to implement a Content Security Policy.
    (_Elevation of Privilage, High Severity_)

- ### Node Admin and Node Wallet Manager only
    **Attacks:**
    CSRF Attacks: `Mitigated` because we avoid using Cookies that could carry sensitive information. Our short lived JWTs are stored in the browser session storage.
    (_Spoofing, High Severity_)

- ### Node Admin Service and Node Wallet Manager Service
    **Description:**
    These Services are the endpoints for the Web Clients of similar name.
    The severtiy of these attacks are lower, as these enpoints are not open to the internet by default but are contained within the local network.
    **Attacks:**
    Credential Stuffing: `Mitigated` because we hash passwords in storage and we plan to use the [Have I Been Pwned](https://haveibeenpwned.com/) API to securly check if a password has been part of a breach. 
    (_Elevation of Privilage, Medium Severity_)
    Credential Cracking: `Mitigated` because we hash passwords that take longer to compare and we plan to implement a rate limiting mechanism.
    (_Elevation of Privilage, High Severity_)

- ### Node
    **Attacks:**
    DDos

- ### Block Service
    **Attacks:** 
    High Complexity Queries: We plan to add size limits to requests to preventing users from executing high workload requests.
    (_Denial of Service, Medium Severity_)