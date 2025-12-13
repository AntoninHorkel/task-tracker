# Task Tracker

## Environment variables

- `REDIS_URL`: Redis URL, defaults to `redis://127.0.0.1`
- `ROUTER_URL`: The router will listen on this URL, defaults to `127.0.0.1:6767`
- `JWT_SECRET`: Self-explanatory

## Authentication Endpoints

### POST `/auth/register`

Register a new user

#### Request Payload

- Type: JSON
- Structure:
    ```json
    {
      "username": <string>,
      "password": <string>
    }
    ```

#### Response Payloads

- HTTP 200 (OK):
    - Type: JSON
    - Structure:
        ```json
        {
          "jwt": <string>
        }
        ```
- HTTP 409 (CONFLICT): <error string>
- HTTP 500 (INTERNAL SERVER ERROR): <error string>

### POST `/auth/login`

Login with username/password or refresh with JWT

#### Request Payload

- Type: JSON
- Structure (Option 1 - Username/Password):
    ```json
    {
      "username": <string>,
      "password": <string>
    }
    ```
- Structure (Option 2 - JWT Refresh):
    ```json
    {
      "jwt": <string>
    }
    ```

#### Response Payloads

- HTTP 200 (OK):
    - Type: JSON
    - Structure:
        ```json
        {
          "jwt": <string>,
          "username": <string>
        }
        ```
- HTTP 400 (BAD REQUEST): <error string>
- HTTP 401 (UNAUTHORIZED): <error string>
- HTTP 500 (INTERNAL SERVER ERROR): <error string>

### POST `/auth/logout`

Logout and revoke JWT

#### Request Payload

- Type: JSON
- Structure:
    ```json
    {
      "jwt": <string>
    }
    ```

#### Response Payloads

- HTTP 200 (OK): No content
- HTTP 401 (UNAUTHORIZED): <error string>
- HTTP 500 (INTERNAL SERVER ERROR): <error string>

## Task Endpoints

### GET `/task`

Get all tasks for authenticated user

#### Request Payload

- Type: JSON
- Structure:
    ```json
    {
      "jwt": <string>
    }
    ```

#### Response Payloads

- HTTP 200 (OK):
    - Type: JSON
    - Structure:
        ```json
        [
          {
            "id": <uuid string>,
            "category": <string>,
            "text": <string>,
            "completed": <bool>,
            "due": <int | null>
          }
        ]
        ```
    - Note: `due` is a UNIX timestamp or null
- HTTP 401 (UNAUTHORIZED): <error string>
- HTTP 500 (INTERNAL SERVER ERROR): <error string>

### POST `/task`

Create a new task

#### Request Payload

- Type: JSON
- Structure:
    ```json
    {
      "jwt": <string>,
      "category": <string>,
      "text": <string>,
      "completed": <bool>,
      "due": <int | null>
    }
    ```
- Note: `due` is a UNIX timestamp or null

#### Response Payloads

- HTTP 201 (CREATED): No content
- HTTP 401 (UNAUTHORIZED): <error string>
- HTTP 500 (INTERNAL SERVER ERROR): <error string>

### GET `/task/{id}`

Get a specific task by ID

#### Path Parameters

- `id`: <uuid string> - The task ID

#### Request Payload

- Type: JSON
- Structure:
    ```json
    {
      "jwt": <string>
    }
    ```

#### Response Payloads

- HTTP 200 (OK):
    - Type: JSON
    - Structure:
        ```json
        {
          "id": <uuid string>,
          "category": <string>,
          "text": <string>,
          "completed": <bool>,
          "due": <int | null>
        }
        ```
    - Note: `due` is a UNIX timestamp or null
- HTTP 401 (UNAUTHORIZED): <error string>
- HTTP 404 (NOT FOUND): <error string>
- HTTP 500 (INTERNAL SERVER ERROR): <error string>

### POST `/task/{id}`

Update an existing task

#### Path Parameters

- `id`: <uuid string> - The task ID

#### Request Payload

- Type: JSON
- Structure:
    ```json
    {
      "jwt": <string>,
      "category": <string | null>,
      "text": <string | null>,
      "completed": <bool | null>,
      "due": <int | null>
    }
    ```
- Note: All fields except `jwt` are optional. Only provided fields will be updated.
- Note: `due` is a UNIX timestamp or null

#### Response Payloads

- HTTP 200 (OK): No content
- HTTP 401 (UNAUTHORIZED): <error string>
- HTTP 404 (NOT FOUND): <error string>
- HTTP 500 (INTERNAL SERVER ERROR): <error string>

### DELETE `/task/{id}`

Delete a task

#### Path Parameters

- `id`: <uuid string> - The task ID

#### Request Payload

- Type: JSON
- Structure:
    ```json
    {
      "jwt": <string>
    }
    ```

#### Response Payloads

- HTTP 200 (OK): No content
- HTTP 401 (UNAUTHORIZED): <error string>
- HTTP 404 (NOT FOUND): <error string>
- HTTP 500 (INTERNAL SERVER ERROR): <error string>
