# 🛡️ ttp-idm

[![MegaLinter](https://github.com/diz-unimr/ttp-idm/actions/workflows/mega-linter.yml/badge.svg)](https://github.com/diz-unimr/ttp-idm/actions/workflows/mega-linter.yml)
[![build](https://github.com/diz-unimr/ttp-idm/actions/workflows/build.yaml/badge.svg)](https://github.com/diz-unimr/ttp-idm/actions/workflows/build.yaml)
[![docker](https://github.com/diz-unimr/ttp-idm/actions/workflows/release.yaml/badge.svg)](https://github.com/diz-unimr/ttp-idm/actions/workflows/release.yaml)
[![codecov](https://codecov.io/gh/diz-unimr/ttp-idm/graph/badge.svg?token=Izcyq8RwyX)](https://codecov.io/gh/diz-unimr/ttp-idm)


> TTP Identity Management service

This service provides identity and pseudonym management with the Trusted third party (TTP) tools E-PIX and gPAS.

## API

### <code>POST</code> <code><b>/api/pseudonyms</b></code> <code>(create pseudonyms for patient)</code>

Adds patient to E-PIX and generate pseudonyms and ids for the provided `study`.

The property `lab_id_count` determines the number of secondary pseudonyms to be created.

#### Parameters

> None

#### Body

> | content-type       | data type   | required |
> |--------------------|-------------|----------|
> | `application/json` | `IdRequest` | true     |

##### IdRequest (JSON Schema)

```json
{
  "type": "object",
  "properties": {
    "first_name": {
      "type": "string"
    },
    "last_name": {
      "type": "string"
    },
    "birth_date": {
      "type": "string",
      "format": "date"
    },
    "birth_place": {
      "type": "string"
    },
    "birth_name": {
      "type": "string"
    },
    "postal_code": {
      "type": "string"
    },
    "city": {
      "type": "string"
    },
    "study": {
      "type": "string"
    },
    "lab_id_count": {
      "type": "number"
    }
  },
  "required": [
    "first_name",
    "last_name",
    "birth_date",
    "birth_place",
    "postal_code",
    "city",
    "study",
    "lab_id_count"
  ]
}
```

#### Responses

> | http code                   | content-type               | response      |
> |-----------------------------|----------------------------|---------------|
> | `200` Ok                    | `application/json`         | `IdResponse`  |
> | `201` Created               | `application/json`         | `IdResponse`  |
> | `500` Internal Server Error | `text/plain;charset=UTF-8` | Error message |

### IdResponse (JSON Schema)

```json
{
  "type": "object",
  "properties": {
    "patient_id": {
      "type": "string"
    },
    "lab_ids": {
      "type": "array",
      "items": {
        "type": "string"
      }
    }
  },
  "required": [
    "patient_id",
    "lab_ids"
  ]
}
```

## Configuration properties

Application properties are read from a properties file ([app.yaml](./app.yaml)) with default values.

| Name                          | Default           | Description                             | Required |
|-------------------------------|-------------------|-----------------------------------------|----------|
| `log_level`                   | info              | Log level (error,warn,info,debug,trace) |          |
| `auth.basic.username`         |                   | Basic auth username for this service    |          |
| `auth.basic.password`         |                   | Basic auth password for this service    |          |
| `ttp.epix.base_url`           |                   | E-PIX base url                          | ✓        |
| `ttp.epix.domain.name`        | test              | E-PIX MPI domain                        | ✓        |
| `ttp.epix.domain.description` | Test domain       | E-PIX MPI domain description            | ✓        |
| `ttp.epix.identifier_domain`  | MPI               | E-PIX MPI identifier domain             | ✓        |
| `ttp.epix.data_source`        | dummy_safe_source | E-PIX id safe source                    | ✓        |
| `ttp.gpas.base_url`           |                   | gPAS base url                           | ✓        |
| `ttp.timeout`                 | 120               | Retry timeout                           |          |

### Environment variables

Override configuration properties by providing environment variables with their respective property names. Replace `.`
with double underscore (`__`).

## Example deployment

Docker compose:

```yaml
query:
  image: ghcr.io/diz-unimr/ttp-idm:latest
  environment:
    LOG_LEVEL: debug
    AUTH__BASIC__PASSWORD: test
    AUTH__BASIC__USERNAME: test
    TTP__EPIX__BASE_URL: http://localhost:8080
    TTP__GPAS__BASE_URL: http://localhost:8081
    TTP__RETRY__COUNT: 3
    TTP__RETRY__TIMEOUT: 5
    TTP__RETRY__WAIT: 2
    TTP__RETRY__MAX_WAIT: 15
```

## License

[AGPL-3.0](https://www.gnu.org/licenses/agpl-3.0.en.html)
