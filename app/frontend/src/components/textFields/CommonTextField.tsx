import { TextField, TextFieldProps } from '@mui/material';

export default function CommonTextField(props: TextFieldProps) {
  return (
    <>
      <TextField sx={{ width: '300px' }} variant="outlined" {...props} />
    </>
  );
}
