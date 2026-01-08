def fizzbuzz(n):
    output = ''
    if n % 3 == 0:
        output += 'Fizz'
    if n % 5 == 0:
        output += 'Buzz'
    if output == '':
        output += f'{n}'
    return output

lis = []
for n in range(10_000):
    lis.append(fizzbuzz(n))
print(lis)
