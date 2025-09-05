import { Password, Visibility, VisibilityOff } from '@mui/icons-material';
import { IconButton, InputAdornment, TextFieldProps } from '@mui/material';
import React from 'react';
import CommonTextField from './CommonTextField';

export default function PasswordField(props: TextFieldProps) {
  const [showPassword, setShowPassword] = React.useState(false);

  return (
    <>
      <CommonTextField
        autoFocus={true}
        aria-label="user-name"
        label="Password"
        type={showPassword ? 'text' : 'password'}
        required
        slotProps={{
          input: {
            startAdornment: (
              <InputAdornment position="start">
                <Password />
              </InputAdornment>
            ),
            endAdornment: (
              <InputAdornment position="start">
                <IconButton
                  aria-label={
                    showPassword ? 'hide the password' : 'display the password'
                  }
                  onClick={() => setShowPassword(!showPassword)}
                  onMouseDown={(e) => e.preventDefault()}
                  onMouseUp={(e) => e.preventDefault()}
                  edge="end"
                >
                  {showPassword ? <VisibilityOff /> : <Visibility />}
                </IconButton>
              </InputAdornment>
            ),
          },
        }}
        {...props}
      />
    </>
  );
}
