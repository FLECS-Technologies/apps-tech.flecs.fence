import { AccountCircle } from '@mui/icons-material';
import { InputAdornment, TextFieldProps } from '@mui/material';
import CommonTextField from './CommonTextField';

export default function UsernameField(props: TextFieldProps) {
  return (
    <>
      <CommonTextField
        autoFocus={true}
        aria-label="user-name"
        label="Username"
        type="text"
        required
        slotProps={{
          input: {
            startAdornment: (
              <InputAdornment position="start">
                <AccountCircle />
              </InputAdornment>
            ),
          },
        }}
        {...props}
      />
    </>
  );
}
