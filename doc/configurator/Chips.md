Supported chips
==============

The `chips` directory keeps the supported chips implementations as crates. Each supported peripheral
must implement the corresponding traits from the `parse` crate.

### TODO 

The approach is not yet considered scalable. The approach this should be moving towards is to have
chip configuration `JSON` files, similar to:

```js
// nrf52833.code.json 
{
	"chip": {
		"init_expr": null,
		"dependencies": null,
		"after_init": null,
		"before_init": null,
		"ident": "auto"
	}
}
```

The `*.code.json` file **could** be generated from procedural macros added in the `chips/<SUPPORTED_CHIP>` crate,
with attributes that translate in the JSON keys. This means introducing the configurator in the TockOS source code,
as a dependency.

```js
// nrf52833.peripherals.json 
{
    "peripherals": [
        {
            "uart": [
                "uart0",
                "uart1"
            ]
        },
		{
			"timer": [
				"rtc"
			]
		}
    ],
    "systick": null
}
```

