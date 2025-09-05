import { Button, ButtonProps } from '@mui/material';

export default function LoginButton(props: ButtonProps) {
  return (
    <>
      <Button
        sx={{ width: '300px' }}
        variant="contained"
        aria-label="login-button"
        loading={false}
        type="submit"
        {...props}
      >
        {' '}
        GO{' '}
      </Button>
    </>
  );
}
