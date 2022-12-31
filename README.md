# AWS S3 TUI

A terminal-based tool for navigating S3 objects.

![s3tui](https://user-images.githubusercontent.com/5222673/210156633-4c43376e-a17d-4a90-87bb-43369cfc98b5.gif)

## Usage

You need to be already logged into AWS in your terminal for this to work:
- Check by running `aws s3 ls`

For the cli to pick up data, it needs to be called with an AWS bucket.

```
s3 --bucket=my-awesome-bucket
```

optionally, you can add a prefix argument as well to further filter objects

```
s3 --bucket=my-awesome-bucket --prefix=my-prefix
```

Functionality if fairly limited. Currently the tool can only navigate objects, and copy the uri path (which is all I need)
