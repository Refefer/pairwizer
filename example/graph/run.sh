# This assumes you have the propagon binary compiled and installed!
propagon articles dehydrate --delim ' ' --features articles.categories

# Node prominance algorithms
propagon articles.edges page-rank --iterations 30 --sink-dispersion all > articles.algo.page-rank
propagon articles.algo.page-rank hydrate --vocab articles.vocab > results.page-rank

# Run the example with different model types
propagon articles.edges label-rank --iterations 30 > articles.algo.label-rank
propagon articles.algo.label-rank hydrate --vocab articles.vocab > results.label-rank

propagon articles.edges lpa --chunks 1 --iterations 5 > articles.algo.lpa
propagon articles.algo.lpa hydrate --vocab articles.vocab > results.lpa

propagon articles.edges mc-cluster --num-attractors 1 --min-cluster-size 5 --clusterer similarity > articles.algo.mc-cluster
propagon articles.algo.mc-cluster hydrate --vocab articles.vocab > results.mc-cluster

# Vec-Prop

# First one only uses the unique ids of the nodes to propagate
propagon articles.edges vec-prop --prior articles.features.id --alpha 0.9 --max-terms 10 > articles.algo.vec-prop.id
propagon articles.algo.vec-prop.id hydrate --vocab articles.vocab > results.vec-prop.id

# This one uses the full categories provided by each article
propagon articles.edges vec-prop --prior articles.features --alpha 0.9 --max-terms 10 > articles.algo.vec-prop
propagon articles.algo.vec-prop hydrate --vocab articles.vocab > results.vec-prop

# Create a semi-supervised feature set
awk 'rand() < 0.5' articles.features > articles.features.semi

# This one uses the partial categories provided by each article
propagon articles.edges vec-prop --prior articles.features.semi --alpha 0.9 --max-terms 10 > articles.algo.vec-prop.semi
propagon articles.algo.vec-prop.semi hydrate --vocab articles.vocab > results.vec-prop.semi
 
# GCS - Unweighted, no local neighbors
propagon articles.edges gcs --dims 5 --landmarks 10 --global-bias 1 --passes 1 --local-embed-fns 20000 > articles.algo.gcs.euc
propagon articles.algo.gcs.euc hydrate --vocab articles.vocab > results.gcs.euc

# GCS - Weighted, some local neighborhood preservation
propagon articles.edges gcs --dims 5 --landmarks 10 --weighting degree --global-bias 0.9 --passes 20 > articles.algo.gcs.weighted.euc
propagon articles.algo.gcs.weighted.euc hydrate --vocab articles.vocab > results.gcs.weighted.euc

# GCS - Poincare embedding, no local neighbors
propagon articles.edges gcs --dims 5 --landmarks 10 --global-bias 1 --space poincare --passes 1 --local-embed-fns 20000 > articles.algo.gcs.poin
propagon articles.algo.gcs.poin hydrate --vocab articles.vocab > results.gcs.poin
