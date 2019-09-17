# aqui a gente simplesmente faz uma matriz identidade "barrada"
# da dimensão do número de vértices do grafo, bem easy

import numpy as np

print("---- Gerador de Grafos Completos em Matriz de Adjacência ----")

n = input("Informe quantidade de vértices: ")
nome = "grafo_"+n+".txt"

arq = open(nome, "w")

grafo = np.zeros((int(n),int(n)), dtype=np.short)
for i in range(0,int(n)):
    for j in range(0,int(n)):
        if i != j:
            grafo[i,j] = 1
print(str(grafo))

for i in range(0,int(n)):
    for j in range(0,int(n)):
        arq.write(str(grafo[i,j])+" ")
    arq.write("\n")

arq.close()

# np.savetxt(nome, grafo, delimiter=' ')

