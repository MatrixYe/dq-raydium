import json

from solana.rpc.api import Client, Signature


def get_transaction_details(sig):
    # Solana RPC endpoint
    url = "https://redoubled-skippet-wbhxrrgkog-dedicated.helius-rpc.com?api-key=98c56f1d-1661-4d3c-a847-4ac73a7e4841"
    client = Client(url)
    signature = Signature.from_string(sig)
    # Fetch the transaction details
    response = client.get_transaction(signature, encoding='jsonParsed', max_supported_transaction_version=0)
    detail = response.to_json()
    return json.loads(detail)


if __name__ == "__main__":
    tx_sig = "3fXEuysz2SXj739sYghBXpbsqNHRWmWEXESiJ8xZGMN9qqwzVeR9u9xJcpMhubZ9MxEPZi5Vg3eraAXYywCGWrUk"
    transaction_details = get_transaction_details(tx_sig)
    print(json.dumps(transaction_details, indent=4))
