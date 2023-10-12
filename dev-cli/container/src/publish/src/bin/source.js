#! /usr/bin/env node

import fs from 'node:fs'
import Bundlr from '@bundlr-network/client'

import { parseTags, uploadWith } from '../main.js'
import { AoContractSourceTags } from '../defaults.js'

/**
 * Implement the uploadWith API such to upload a single file
 * to Bundlr
 */
const uploadHyperbeamContractSource = uploadWith({
  walletExists: async (path) => fs.existsSync(path),
  /**
   * implement to check if single file exists
   */
  artifactExists: async (path) => fs.existsSync(path),
  /**
   * implement to read the wallet as JSON from the path
   */
  readWallet: async (path) => JSON.parse(fs.readFileSync(path).toString()),
  /**
   * implement to upload a single file to Arweave
   */
  upload: ({ path, wallet, to: bundlrNode, ...rest }) => {
    const bundlr = new Bundlr(bundlrNode, 'arweave', wallet)
    return bundlr.uploadFile(path, rest)
  }
})

/**
 * The ao cli publish command ultimately executes this
 * code.
 *
 * It expects a wallet JWK to be present in the provided directory
 * in order to perform the upload to Arweave via Bundlr
 */
uploadHyperbeamContractSource({
  walletPath: process.env.WALLET_PATH,
  artifactPath: process.env.CONTRACT_WASM_PATH,
  to: process.env.BUNDLR_NODE || 'https://node2.bundlr.network',
  tags: [
    ...parseTags(process.env.TAGS || ''),
    // Add the proper tags for ao contract source
    ...AoContractSourceTags
  ]
})
  // log transaction id
  .then(console.log)
  .catch((err) => {
    switch (err.code) {
      case 'WalletNotFound': {
        console.error(
          'Wallet not found at the specified path. Make sure to provide the path to your wallet with -w'
        )
        return process.exit(1)
      }
      case 'ArtifactNotFound': {
        console.error(
          'Contract Wasm source not found at the specified path. Make sure to provide the path to your built Wasm'
        )
        return process.exit(1)
      }
      default: {
        throw err
      }
    }
  })
