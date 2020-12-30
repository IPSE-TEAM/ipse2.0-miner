### /api/v0/order/`<address>`

upload data

Methods
***
**`POST`**

**REQUEST QUERY PARAMETERS**

> address: user address
>
> data: uploading data


### /api/v0/order/`<address>`/`hash`


add data info(label,category, describe, days)


Methods
***

**`POST`**

**REQUEST QUERY PARAMETERS**

> cid: user address
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


### /api/v0/order/`<address>`/`<hash>`


delete data

Methods
***

**`DELETE`**

**REQUEST QUERY PARAMETERS**

> address: user address


### /api/v0/order/verify/`<address>`/`<hash>`

verify data 

Methods

***
**`POST`**

**REQUEST QUERY PARAMETERS**

> address: user address
>
> hash:data hash 

**REQUEST BODY**

```
{
    address: String,
    name: String,
    hash: String,
    size: String,
    cumulative_size: String,
    blocks: String,
}
```

