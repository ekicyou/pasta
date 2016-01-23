export default class Cat {
  constructor(name) {
    this.name = name
  }

  meow() {
    return this.name + ': "meow!"';
  }
}
