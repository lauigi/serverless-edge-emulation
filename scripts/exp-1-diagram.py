import matplotlib.pyplot as plt

x_data = [1000, 10000, 100000, 1000000, 2000000, 4000000, 6000000, 8000000, 10000000]
y_data = [15871.2, 14991.85, 10270.8, 8273.9, 6004.75, 8179.75, 8377.6, 1373.95, 1107]

plt.plot(x_data, y_data, marker='o')
plt.xlabel('Number of Entries')
plt.ylabel('Processing Rate')
plt.title('Replication Experiment 1 Attempt 3')
plt.grid(True)
plt.show()