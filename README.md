# lindera-server
A Japanese Morphological Analysis Server.


## Run server

```
% cargo run -- --host=0.0.0.0 --port=3333
```

## Tokenize text
```
% curl -XGET http://localhost:3333 | jq .
```

```json
{
  "tokens": [
    "すもも",
    "も",
    "もも",
    "も",
    "もも",
    "の",
    "うち"
  ]
}
```