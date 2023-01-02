# AWS S3 TUI

A terminal-based tool for navigating S3 objects.

![s3tui](https://user-images.githubusercontent.com/5222673/210156633-4c43376e-a17d-4a90-87bb-43369cfc98b5.gif)

## Usage

You need to be already logged into AWS in your terminal for this to work:
- Check by running `aws s3 ls`

For the cli to pick up data, it needs to be called with an AWS bucket.

```
s3_tui --bucket=my-awesome-bucket
```

optionally, you can add a prefix argument as well to further filter objects

```
s3_tui --bucket=my-awesome-bucket --prefix=my-prefix
```

Functionality if fairly minimal at the moment, since it works well for my personal purposes. Currently, you can:
- Use the arrow keys to navigate around a bucket
- Use the search bar to filter down objects by string matching
- Press "c" when the cursor is on a object of interest to **copy it's URI**

Feel free to raise an issue if more functionality is desired!
