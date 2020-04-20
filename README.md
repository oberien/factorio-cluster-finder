# Factorio Cluster Finder

This program is an experiment to find clusters (subfactories) in the item dependency graph of the game Factorio.
Given an item or a list of items, this program tries to find a cluster around those items with all items that should
comprise a subfactory, producing all of those items.
The dependency graph is based on 0.17.60.

# Results

These are some preliminary discoveries we found, without further analysis (for now?).

* The oil factory should include water, petroleum-gas, steam, crude-oil, coal, light-oil, heavy-oil, sulfur,
  plastic-bar, solid-fuel, explosives, lubricant and rocket-fuel.
* However, sulfuric-acid should be produced on-demand in the subfactories requiring it.
* The transition from heavy-oil to lubricant is redundant and possibly a useless game addition.
  It's a one-to-one transformation from heavy oil to lubricant, without any further requirements.
  Instead, maybe heavy-oil could be used directly in the recipes requiring lubricant instead of it.
  This finding is because lubricant is both in the oil-refinery cluster, as well as in each cluster producing any
  item consuming lubricant.
  Therefore, this conversion process is redundant.
* If a cluster includes iron-plates, copper-plates, stone-bricks or steel, then it will always also include the
  respective ore.
  Considering the ratios of miners and furnaces, and belt item throughput, it appears that the developers of factorio
  intend a large smeltery to be built.
  That's also how most, if not all factories I've seen are built and designed.
  However, as this algorithm always includes the ores in the cluster, it might make sense to actually have furnaces
  inline in each subfactory, feeding only only ores (and oil products) to the subfactories.
* Whenever a cluster around an early item consuming iron plates is found, the algorithm will most likely merge
  iron-plates into the subfactory.
  Having iron plates, all items that can be produced just from iron-plates will also be merged, as they can then
  also trivially be included in that subfactory.
  Similarly, if copper-plates are included in a custer, all items stemming from copper plates will be added.
  The same happens if both iron-plates and copper-plates are in the cluster.
  
# Algorithm

The algorithm used is a trivial greedy search over the dependency graph.
It works on two rules:
* Try to minimize the sum of different inputs and

# Restrictions

The algorithm is probably biased towards some expected findings.
Additionally, the item dependency graph is not perfect.
Among other problems, it doesn't differentiate between multiple different recipes.
For example both light and heavy oil cracking as well as coal liquefaction are represented as a "single, merged recipe".
Thus, petroleum-gas is assumed to have all of coal, crude-oil, light-oil, steam and water as dependency.
Similarly, solid-fuel depends on all of heavy-oil, light-oil *and* petroleum-gas.
