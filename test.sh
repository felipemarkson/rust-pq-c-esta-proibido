test(){
echo "Credito $1"
curl -i -X POST localhost:9999/clientes/$1/transacoes --data "@c.json"
echo ""

echo "Debito $1"
curl -i -X POST localhost:9999/clientes/$1/transacoes --data "@d.json"
echo ""

echo "Extrato $1"
curl -i -X GET localhost:9999/clientes/$1/extrato
echo ""
}


test 1
# test 2
# test 3
# test 4
# test 5
# test 6