# openfpga-instance-packager
A tool for building instance.json files

## Download

If you're looking for the binary you can grab the one for your platform from [The Latest Release](https://github.com/neil-morrison44/openfpga-instance-packager/releases/latest)

It's run as `[binary] path/to/pocket/root` & there'll be an interactrive process of picking which cores you want to build json files for.

Pocket sync uses this library as a dependency so the behaviour of clicking the `Instance JSONs` button in the `Games` view is the exact same.

It's expected that the other updaters will have this functionality built in, so the binary is available as a reference & for anyone who doesn't want to run an updater.

## instance-packager.json format

The `instance-packager.json` instructs updaters etc on how to build `<instance>.json` files [(see Analogue's docs here)](https://www.analogue.co/developer/docs/core-definition-files/instance-json)

### Typing

A Typescript type for the JSON:

```ts

type InstancePackagerJSON = {
  output: string,
  platform_id: string,
  data_slots: {
    // when sort is single this will be the id of the file, or the files will be given ids from id -> id+1 -> id+2 etc
    id: number,
    //glob format so `named_file.bin` * `*.bin` both work
    filename: string,
    // single for single files, asc / des for multiple files will choose between `file 1.bin, file 2.bin` (ascending) and `file 2.bin, file 1.bin` (descending)
    sort: "single" | "ascending" | "descending",
    // will use the name of this file as the output JSON name, otherwise defaults to the name of the folder
    as_filename?: boolean
    // will ignore folders which don't have matching files when doing a search over the Assets folders
    required: boolean
  }[],
  // Gets passed through to the output json as is
  memory_writes?: { data: string | number, address: string | number }[],
  // Gets passed through to the output json as is
  core_select?: { id: number, select: boolean },

  // allows specifying specific values for a certain title, with values fully replacing the root ones
  overrides: {
    [folder_name: string]: {
      // Allows for setting a filename for the output json directly, `"Game Title"` will result in `Game Title.json`
      filename: string,
      data_slots: {
        id: number,
        filename: string,
        sort: "single" | "ascending" | "descending",
        as_filename?: boolean
        required: boolean
      }[],
      memory_writes?: { data: string | number, address: string | number }[],
      core_select?: { id: number, select: boolean },
    }
  }
}

```


### Examples

The most basic format, for a core which needs to create instance.json files for bin & cue files follows:

```json
{
  "output": "Assets/pcecd/Mazamars312.PC Engine CD",
  "platform_id": "pcecd",
  "slot_limit": {
    "count": 27,
    "message": "This message will be shown to the user when the slots in the instance file are > count."
  },
  "data_slots": [
    {
      "id": 100,
      "filename": "*.cue",
      "sort": "single",
      "required": true,
      "as_filename": true
    },
    {
      "id": 101,
      "filename": "*.bin",
      "sort": "ascending",
      "required": true
    }
  ]
}
```

And a more intense example, using the overrides:

```json
{
  "output": "Assets/ng/Mazamars312.NeoGeo",
  "platform_id": "ng",
  "slot_limit": {
      "count": 27,
      "message": "This message will be shown to the user when the slots in the instance file are > count."
   },
  "data_slots": [
    {
      "id": 3,
      "filename": "srom",
      "sort": "single",
      "required": true
    },
    {
      "id": 4,
      "filename": "prom",
      "sort": "single",
      "required": true
    }
  ],
  "memory_writes": [
    {
      "address": "0x00000004",
      "data": "0x12345678"
    }
  ],
  "overrides": {
    "wjammss": {
      "file_name": "Windjammers",
      "memory_writes": [
        {
          "address": "0x00000004",
          "data": "0x87654321"
        }
      ],
      "data_slots": [
        {
          "id": 3,
          "filename": "srom",
          "sort": "single",
          "required": true
        },
        {
          "id": 4,
          "filename": "prom",
          "sort": "single",
          "required": true
        },
        {
          "id": 5,
          "filename": "crom0",
          "sort": "single",
          "required": true
        }
      ]
    }
  }
}
```
