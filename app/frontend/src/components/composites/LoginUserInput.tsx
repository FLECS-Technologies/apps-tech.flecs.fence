import Grid from '@mui/material/Grid';
import UsernameField from '../textFields/UsernameField';
import PasswordField from '../textFields/PasswordField';
import LoginButton from '../buttons/LoginButton';
import { useState } from 'react';
import { login } from '@services/auth/login';

export default function LoginUserInput() {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [loggingIn, setLoggingIn] = useState(false);

  return (
    <>
      <Grid container columns={12} spacing={2}>
        <Grid size={12}>
          <UsernameField onChange={(e) => setUsername(e.currentTarget.value)} />
        </Grid>
        <Grid size={12}>
          <PasswordField onChange={(e) => setPassword(e.currentTarget.value)} />
        </Grid>
        <Grid size={12}>
          <LoginButton
            disabled={!username || !password}
            loading={loggingIn}
            onClick={async () => {
              setLoggingIn(true);
              await login(username, password);
              setLoggingIn(false);
            }}
          />
        </Grid>
      </Grid>
    </>
  );
}
