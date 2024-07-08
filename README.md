# world-wonders-api

[![Docker Image Size](https://img.shields.io/docker/image-size/rolvapneseth/world-wonders-api?label=Docker%20image)](https://hub.docker.com/r/rolvapneseth/world-wonders-api)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/Rolv-Apneseth/world-wonders-api/prod.yml)](https://github.com/Rolv-Apneseth/world-wonders-api/actions/workflows/prod.yml)
[![License](https://img.shields.io/badge/License-AGPLv3-green.svg)](./LICENSE)

Free, open and self-hostable API providing information about [Wonders of the World](https://en.wikipedia.org/wiki/Wonders_of_the_World).

## Description

This is an API which serves information about various of the most famous wonders from around the world,
with information such as when a wonder was built, where it's located, and even a collection of images
and various other links related to it.

A public instance is available for anyone to use at [world-wonders-api.org](https://www.world-wonders-api.org), no API keys are required. This is subject to change if it is
deemed necessary in the future, but hopefully that won't be the case.

Documentation is available [here](https://www.world-wonders-api.org/v0/docs).

## Deployment

You can self-host using Docker:

```bash
docker run -d -p 8138:8138 \
  --name world-wonders-api rolvapneseth/world-wonders-api
```

If you prefer using `docker compose`, save this as your `docker-compose.yml` file and run `docker compose up`:

```yaml
version: "3"
services:
  api:
    image: rolvapneseth/world-wonders-api
    ports:
      - "8138:8138"
```

Then access the documentation on your local machine at [http://0.0.0.0:8138/v0/docs](http://0.0.0.0:8138/v0/docs).

## Responses

All data responses are in the [JSON](http://json.org/) format.

### Example of a successful response

```json
{
  "name": "Great Pyramid of Giza",
  "location": "Giza, Egypt",
  "build_year": -2560,
  "time_period": "Ancient",
  "links": {
    "wiki": "https://en.wikipedia.org/wiki/Great_Pyramid_of_Giza",
    "britannica": "https://www.britannica.com/place/Great-Pyramid-of-Giza",
    "google_maps": "https://www.google.com/maps/place/The+Great+Pyramid+of+Giza/...",
    "trip_advisor": "https://www.tripadvisor.com/Attraction_Review-g294202-d...",
    "images": [
      "https://upload.wikimedia.org/wikipedia/commons/e/e3/Kheops-Pyramid.jpg",
      "https://cdn.britannica.com/75/178475-050-E9212E3D/Pyramid-of-Khufu-Giza-Egypt.jpg",
      "https://cdn.britannica.com/06/122506-050-C8E03A8A/Pyramid-of-Khafre-Giza-Egypt.jpg",
    ]
  },
  "categories": [
    "SevenWonders",
  ]
}
```

### Example of an error response

The response contain a non `200` HTTP status code, such as `400`, with a body like this:

```json
{
  "message": "The provided lower limit of 1000 is greater than the provided upper limit of 400"
}
```

## Thanks

This API was originally inspired by and contains some data gathered from [this dataset on kaggle](https://www.kaggle.com/datasets/karnikakapoor/wonders-of-world).

## License

[AGPL-3.0](./LICENSE)
