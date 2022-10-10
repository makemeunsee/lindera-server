# lindera-server

A Japanese Morphological Analysis Server.

## Run server

```
% cargo run --features ipadic -- -H 0.0.0.0 -p=3333 -t ipadic
```

## Tokenize text

```
% curl -XPOST -H 'Content-type: text/plain' http://localhost:3333/tokenize --data-binary 'すもももももももものうち' | jq .
```

```json
[
  {
    "detail": [
      "名詞",
      "一般",
      "*",
      "*",
      "*",
      "*",
      "すもも",
      "スモモ",
      "スモモ"
    ],
    "text": "すもも"
  },
  {
    "detail": [
      "助詞",
      "係助詞",
      "*",
      "*",
      "*",
      "*",
      "も",
      "モ",
      "モ"
    ],
    "text": "も"
  },
  {
    "detail": [
      "名詞",
      "一般",
      "*",
      "*",
      "*",
      "*",
      "もも",
      "モモ",
      "モモ"
    ],
    "text": "もも"
  },
  {
    "detail": [
      "助詞",
      "係助詞",
      "*",
      "*",
      "*",
      "*",
      "も",
      "モ",
      "モ"
    ],
    "text": "も"
  },
  {
    "detail": [
      "名詞",
      "一般",
      "*",
      "*",
      "*",
      "*",
      "もも",
      "モモ",
      "モモ"
    ],
    "text": "もも"
  },
  {
    "detail": [
      "助詞",
      "連体化",
      "*",
      "*",
      "*",
      "*",
      "の",
      "ノ",
      "ノ"
    ],
    "text": "の"
  },
  {
    "detail": [
      "名詞",
      "非自立",
      "副詞可能",
      "*",
      "*",
      "*",
      "うち",
      "ウチ",
      "ウチ"
    ],
    "text": "うち"
  }
]
```
