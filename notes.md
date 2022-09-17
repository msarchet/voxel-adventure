Currently trying to make structures work by tracking a secondary u64 per voxel 

0 - 8: Cave Generation Data
9 - 16: Decorator Data

When Generating structures such as caves, decorators, buildings etc...

A chunk is asked to be generated to be rendered. To generate the
A Chunk is given a component called `GenerateStructure(StructureType)` or similar. A system 