#! /bin/sh -eu

# Execution command
# ($ cd PCT)
# $ script/ex/ex1.sh

timestampNow () {
    echo $(date "+%s")
}

filename () {
    echo "data/result/journal/output-$1-$2-$3-$4-$5-$(timestampNow).txt"
}

## using FSA and gp10
DS=fsa
EN=gp10
make clean && FEATURE="$DS $EN" make
for chunksize in 1000000 10000000 20000000
do
    echo "start $DS $EN $chunksize"
    bin/app $chunksize data/real/client/gp10/gp10-client-1000-20201118110924.json data/real/central/gp10/gp10-central-10000000-20201119020253.json > $(filename $DS $EN $chunksize 1000 1000)
    bin/app $chunksize data/real/client/gp10/gp10-client-3000-20201118112208.json data/real/central/gp10/gp10-central-30000000-20201119020444.json > $(filename $DS $EN $chunksize 3000 3000)
    bin/app $chunksize data/real/client/gp10/gp10-client-5000-20201118114542.json data/real/central/gp10/gp10-central-50000000-20201119020812.json > $(filename $DS $EN $chunksize 5000 5000)
done

## using HashTable and gp10
DS=hashtable
EN=gp10
make clean && FEATURE="$DS $EN" make
for chunksize in 1000000 10000000 20000000
do
    echo "start $DS $EN $chunksize"
    bin/app $chunksize data/real/client/gp10/gp10-client-1000-20201118110924.json data/real/central/gp10/gp10-central-10000000-20201119020253.json > $(filename $DS $EN $chunksize 1000 1000)
    bin/app $chunksize data/real/client/gp10/gp10-client-3000-20201118112208.json data/real/central/gp10/gp10-central-30000000-20201119020444.json > $(filename $DS $EN $chunksize 3000 3000)
    bin/app $chunksize data/real/client/gp10/gp10-client-5000-20201118114542.json data/real/central/gp10/gp10-central-50000000-20201119020812.json > $(filename $DS $EN $chunksize 5000 5000)
done

## using FSA and th48
DS=fsa
EN=th48
make clean && FEATURE="$DS $EN" make
for chunksize in 1000000 10000000 20000000 50000000
do
    echo "start $DS $EN $chunksize"
    bin/app $chunksize data/real/client/th48/th48-client-1000-20201118093934.json data/real/central/th48/th48-central-10000000-20201119014912.json > $(filename $DS $EN $chunksize 1000 1000)
    bin/app $chunksize data/real/client/th48/th48-client-3000-20201118095337.json data/real/central/th48/th48-central-30000000-20201119015105.json > $(filename $DS $EN $chunksize 3000 3000)
    bin/app $chunksize data/real/client/th48/th48-client-5000-20201118101852.json data/real/central/th48/th48-central-50000000-20201119015438.json > $(filename $DS $EN $chunksize 5000 5000)
done

## using HashTable and th48
DS=hashtable
EN=th48
make clean && FEATURE="$DS $EN" make
for chunksize in 1000000 10000000 20000000 50000000
do
    echo "start $DS $EN $chunksize"
    bin/app $chunksize data/real/client/th48/th48-client-1000-20201118093934.json data/real/central/th48/th48-central-10000000-20201119014912.json > $(filename $DS $EN $chunksize 1000 1000)
    bin/app $chunksize data/real/client/th48/th48-client-3000-20201118095337.json data/real/central/th48/th48-central-30000000-20201119015105.json > $(filename $DS $EN $chunksize 3000 3000)
    bin/app $chunksize data/real/client/th48/th48-client-5000-20201118101852.json data/real/central/th48/th48-central-50000000-20201119015438.json > $(filename $DS $EN $chunksize 5000 5000)
done