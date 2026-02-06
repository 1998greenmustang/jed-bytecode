FIZZBUZZ = "FizzBuzz"
FIZZ = "Fizz"
BUZZ = "Buzz"

lis = [x for x in range(1, 10001)]

def fizzbuzz(x):
        divbythree = x % 3
        divbyfive = x % 5

        if divbythree and divbyfive:
            return FIZZBUZZ
        if divbythree:
            return FIZZ
        if divbyfive:
            return BUZZ
        return x

lise = [fizzbuzz(lis[x]) for x in range(len(lis))]
print(lise)


