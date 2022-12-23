# mdtransform
This is a simple program that renders Markdown as HTML for websites.

You can use the title directive to set a page's title:
```md
# TITLE: your title here
```
becomes
```html
<title>your title here</title>
<center>
    <h1 style='margin-bottom: 0px; font-size: 2.5rem;'>your title here</h1>
    <hr />
</center>
```