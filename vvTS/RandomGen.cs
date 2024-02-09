using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000102 RID: 258
	[HandlerCategory("vvTrade"), HandlerName("Случайное число")]
	public class RandomGen : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x0600076C RID: 1900 RVA: 0x00020A68 File Offset: 0x0001EC68
		public IList<double> Execute(IList<double> src)
		{
			int count = src.Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = this._r.NextDouble();
			}
			return array;
		}

		// Token: 0x17000261 RID: 609
		[HandlerParameter(true, "0")]
		public int Seed
		{
			// Token: 0x0600076A RID: 1898 RVA: 0x00020A3C File Offset: 0x0001EC3C
			get
			{
				return this._seed;
			}
			// Token: 0x0600076B RID: 1899 RVA: 0x00020A44 File Offset: 0x0001EC44
			set
			{
				this._seed = value;
				this._r = ((value == 0) ? new Random() : new Random(this._seed));
			}
		}

		// Token: 0x04000277 RID: 631
		private Random _r = new Random();

		// Token: 0x04000278 RID: 632
		private int _seed;
	}
}
