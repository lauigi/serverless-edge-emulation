import matplotlib.pyplot as plt

# Define the data
clients = [1, 2, 4, 6, 8, 10, 12]
edge_li = [517.64, 522.595, 621.7, 818.945, 1026.505, 1232.72, 1386.065]
edge_rp = [524.25, 538.145, 580.075, 661.41, 765.145, 879.205, 1029.23]
edge_rr = [529.34,529.925,595.775,650.315,790.46,928.405,1108.405]
distributed = [519.52, 520.6, 520.5, 519.72, 519.815, 521.3, 520.915]

# Create the plot
plt.figure(figsize=(10, 6))

# Plot each line with different markers
plt.plot(clients, edge_li, 'o-', label='Edge/LI')
plt.plot(clients, edge_rp, 's-', label='Edge/RP')
plt.plot(clients, edge_rr, '^-', label='Edge/RR')
plt.plot(clients, distributed, 'd-', label='Distributed')

# Customize the plot
plt.xlabel('Number of clients')
plt.ylabel('95th percentile of delay (ms)')
plt.grid(True, linestyle='--', alpha=0.7)
plt.legend()
plt.title('Replication Grid scenario (with emulated e-computers): 95th percentile of delay vs. number of clients.')

# Set axis limits
plt.ylim(500, 1200)
plt.xlim(0, 13)

plt.show()
