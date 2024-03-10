echo "Credito"
curl -i -X POST localhost:9999/clientes/1/transacoes --data "@c.json"

echo "Debito"
curl -i -X POST localhost:9999/clientes/1/transacoes --data "@d.json"

echo "Extrato"
curl -i -X GET localhost:9999/clientes/1/extrato

echo "Invalid Get"
curl -i -X GET localhost:9999/clientes/1/transacoes

echo "Invalid Post"
curl -i -X POST localhost:9999/clientes/1/extrato