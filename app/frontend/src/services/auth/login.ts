import axios, { AxiosError } from 'axios';

export interface LoginRequest {
  username: string;
  password: string;
}

export async function login(username: string, password: string): Promise<void> {
  const endpoint = 'http://localhost:8960/auth/login';

  const requestData: LoginRequest = {
    username,
    password,
  };

  return await axios
    .post(endpoint, requestData)
    .then(() => Promise.resolve())
    .catch((err) => {
      if (err instanceof AxiosError) {
        if (err.response && err.response.status == 403) {
          Promise.reject('Invalid username and/or password');
        }
        if (err.response && err.response.data) {
          Promise.reject(err.response.data);
        }
      }
      Promise.reject(err);
    });
}
