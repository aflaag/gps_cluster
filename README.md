# gps-cluster

This small program will take some pictures in input, and based on the metadata on every image, it will group them
by their GPS position, if available.

Starting from the given folder, the program will **recursively** go through any picture and find "clusters"
of images by using their location. Then, it will generate a folder for each cluster, named with the coordinates
of the rough location of the cluster.

The clusters are generated by using the `<THRESHOLD VALUE>` (see **Usage** section), which represents the radius,
in **meters**, used to group the images.

## Usage

To use this program, run

```sh
gps_cluster --input <INPUT FOLDER> --output <OUTPUT FOLDER> --threshold <THRESHOLD VALUE>
```

where `<INPUT FOLDER>` is the input folder, which can contain other folders since *the program works
recursively*, `<OUTPUT FOLDER>` is the output directory, which **must exist** and **must be empty**, and lastly
`THRESHOLD VALUE` is the radius, in **meters**, used to generate the clusters.
Also, a `--verbose` flag is available, for a better console output.

