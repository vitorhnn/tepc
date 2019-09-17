class Graph():
    def __init__(self, n):  # n = número de vértices
        self.n = n
        self.graph = {}  # um dicionário
        self.Marcado = []

    def N(self, v):  # v = um vértice v
        return self.graph[v]  # retorno a lista de vértices adjacentes ao v

    def addEdges(self, v, w):
        self.graph[v].append(w)
        self.graph[w].append(v)


