import { Typography } from '@mui/material';

import openSourceTxt from '../assets/third-party-licenses.txt';
import { useEffect, useState } from 'react';

export default function OpenSource() {
  console.log(openSourceTxt);

  const [content, setContent] = useState('');
  useEffect(() => {
    fetch(openSourceTxt)
      .then((response) => response.text())
      .then(setContent);
  }, []);

  return (
    <>
      <Typography variant="h4">Open Source</Typography>
      <Typography sx={{ whiteSpace: 'pre-wrap', textAlign: 'left', p: 4 }}>
        {content}
      </Typography>
    </>
  );
}
