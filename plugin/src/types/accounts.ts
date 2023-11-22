interface DevnetAccount {
  initial_balance: number
  address: string
  private_key: string
}

interface ManualAccount {
  address: string
  private_key: string
  public_key: string
  balance: string
}

export type { DevnetAccount, ManualAccount }
