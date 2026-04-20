# 🚗 Oldtimers XRPL NFT RWA Microservice

**XRPL-based microservice for minting and managing Real World Asset (RWA) “Vehicle Passport” NFTs using real-time vehicle metadata.**

---

## 🌐 Overview

The **Oldtimers XRPL NFT RWA Microservice** is a dedicated backend service responsible for bridging real-world vehicle data with the XRP Ledger by minting NFTs that represent physical assets.

Each NFT acts as a **Vehicle Passport** — a verifiable, on-chain representation of a classic vehicle, containing structured metadata and references to off-chain media.

This microservice is a core component of the **Oldtimers Offer** ecosystem, enabling scalable and secure tokenization of real-world assets.

---

## 🎯 Purpose

This service enables:

- 🚗 Tokenization of classic vehicles as NFTs  
- 🔗 Linking real-world assets with blockchain records  
- 🛡️ Controlled and verified NFT issuance  
- ⚡ Fast and low-cost minting using XRPL  

---

## 🧠 RWA Concept

In the context of **Real World Asset (RWA) tokenization**, this microservice transforms:

**Physical Vehicle → Structured Metadata → XRPL NFT**

Each NFT contains:
- Vehicle identity (make, model, year)
- Technical attributes (color, mileage, type)
- Media links (hosted via CDN)
- External reference to the marketplace listing

This creates a **trusted digital layer** for real-world assets.

---

## ⚙️ Core Features

### 🔹 XRPL NFT Minting
- Native XRPL NFT minting (no smart contracts required)
- Fast finality and low transaction costs

### 🔹 Dynamic Metadata Generation
- Fetches real-time vehicle data from the Rust backend
- Automatically builds NFT metadata (JSON → URI encoding)

### 🔹 Secure API Access
- Protected endpoints using API keys / bearer tokens
- Prevents unauthorized minting

### 🔹 Transaction Tracking
- Returns NFT ID and transaction hash
- Enables verification via XRPL explorers

### 🔹 Modular Microservice Design
- Fully decoupled from main backend
- Can scale independently

---

## 🔄 Minting Flow

```text
[User Lists Vehicle]
        ↓
[Admin Approves Listing]
        ↓
[WordPress Plugin → Mint Request]
        ↓
[XRPL Microservice]
        ↓
[Fetch Vehicle Metadata (Rust Backend)]
        ↓
[Generate NFT Metadata JSON]
        ↓
[Mint NFT on XRPL]
        ↓
[Return NFT ID + TX Hash]
        ↓
[Display NFT Badge in Frontend]
```

## 🏗️ Architecture Role

The XRPL NFT RWA Microservice serves as a critical component within the Oldtimers Offer ecosystem, acting as a bridge between traditional Web2 infrastructure and Web3 blockchain systems.

### 🔗 Responsibilities

- **Web2 → Web3 Bridge**
  - Connects the Rust backend and WordPress admin interface with the XRPL network  
  - Translates real-world vehicle data into blockchain-compatible NFT structures  

- **NFT Minting Engine**
  - Handles the full lifecycle of NFT creation on XRPL  
  - Generates metadata, encodes it, and submits mint transactions  

- **Controlled Tokenization Layer**
  - Ensures that only verified and approved vehicles are tokenized  
  - Integrates with admin-controlled workflows to prevent unauthorized minting  

- **Decoupled Microservice**
  - Operates independently from the main backend  
  - Enables horizontal scalability and isolated deployment  

### 🧩 System Integration

The service integrates with:

- **Rust (Actix Web) Backend** → provides real-time vehicle metadata  
- **WordPress Admin Plugin** → triggers NFT minting actions  
- **Yew Frontend** → displays NFT status and verification badges  
- **Cloud Storage / CDN** → hosts images referenced in NFT metadata  

---

## 🔗 XRPL Integration

This microservice leverages the XRP Ledger (XRPL) to enable efficient and scalable Real World Asset (RWA) tokenization through NFTs.

### ⚡ Why XRPL

- **Low Transaction Costs**
  - Ideal for large-scale asset tokenization  

- **Fast Finality**
  - Transactions are confirmed within seconds  

- **Native NFT Support (XLS-20)**
  - No need for complex smart contracts  

- **Energy Efficient**
  - Suitable for sustainable Web3 infrastructure  

---

### 🪙 NFT Minting Process

- Constructs NFT metadata from real-time vehicle data  
- Encodes metadata into a URI-compatible format  
- Submits mint transaction to XRPL  
- Receives:
  - `nft_id` (unique identifier)
  - `tx_hash` (transaction reference)

---

### 🔍 Verification

Each minted NFT can be publicly verified via XRPL explorers:

- https://livenet.xrpl.org/  
- https://bithomp.com/  

This ensures full transparency and trust between the physical asset and its digital representation.

---

### 🌐 Role in RWA Tokenization

XRPL acts as the **on-chain layer** that:

- Stores immutable references to real-world vehicles  
- Enables verifiable ownership and authenticity  
- Provides a foundation for future use cases such as:
  - Ownership tracking  
  - Marketplace validation  
  - Integration with financial services  
