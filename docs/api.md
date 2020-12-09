### /api/v0/order/`<cid>`

upload data

Methods
***
**`POST`**

**REQUEST QUERY PARAMETERS**

> cid: user public_key
>
> data: uploading data


### /api/v0/order/`<cid>`/`hash`


add data info(label,category, describe, days)


Methods
***

**`POST`**

**REQUEST QUERY PARAMETERS**

> cid: user public_key
>
> hash: data hash 

**REQUEST BODY**

```
{
	"label": String,
	"category": String,
	"describe": String,
	"days": Int,
}
```


### /api/v0/order/`<cid>`/`<hash>`


delete data

Methods
***

**`DELETE`**

**REQUEST QUERY PARAMETERS**

> cid: user public_key


### /api/v0/order/verify/`<cid>`/`<hash>`

verify data 

Methods

***
**`POST`**

**REQUEST QUERY PARAMETERS**

> cid: user public_key
>
> hash:data hash 

**REQUEST BODY**

```
{
    cid: String,
    name: String,
    hash: String,
    size: String,
    cumulative_size: String,
    blocks: String,
}
```

