# Converter Contract

The Converter Contract contains the logic for swapping stLuna/bLuna tokens with the same API as [Astroport's native pool
contract](https://github.com/astroport-fi/astroport-core/tree/master/contracts/pair#executemsg), but it's just simply calls [Hub::Convert](https://docs.terra.lido.fi/contracts/hub#convert) under the hood.
