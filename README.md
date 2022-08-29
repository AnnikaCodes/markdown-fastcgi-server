# markdown-fastcgi-server
This is a FastCGI server that renders Markdown as HTML. I created this for a personal website but figured it might help others. Change `HTML_PREFIX` in `src/main.rs` if you don't want my website linked in each render.

Example NGINX configuration looks like this (you should probably cache things in addition):
```nginx
location / {
    # Rewrite URLs with no extention to .md (can't be a try_files)
    rewrite '/([^.]{1,})$' /$1.md;
    try_files $uri $uri.html $uri/ =404;

    fastcgi_param FILE $request_filename;
    if ($uri ~ \.md$) {
        # Server runs on port 9000
        fastcgi_pass localhost:9000;
    }
}
```