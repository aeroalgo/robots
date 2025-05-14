using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200010F RID: 271
	[HandlerCategory("vvTrade"), HandlerName("Нормализатор [0..1]")]
	public class Normalizer01 : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600079C RID: 1948 RVA: 0x00021734 File Offset: 0x0001F934
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("normalized01", new string[]
			{
				this.NormalizationPeriod.ToString(),
				src.GetHashCode().ToString()
			}, () => Normalizer01.GenNormalizer01(src, this.NormalizationPeriod, this.Context));
		}

		// Token: 0x0600079B RID: 1947 RVA: 0x000215CC File Offset: 0x0001F7CC
		public static IList<double> GenNormalizer01(IList<double> _price, int _NormPeriod, IContext _ctx)
		{
			int count = _price.Count;
			double[] array = new double[count];
			IList<double> data = _ctx.GetData("llv", new string[]
			{
				_NormPeriod.ToString(),
				_price.GetHashCode().ToString()
			}, () => Series.Lowest(_price, _NormPeriod));
			IList<double> data2 = _ctx.GetData("hhv", new string[]
			{
				_NormPeriod.ToString(),
				_price.GetHashCode().ToString()
			}, () => Series.Highest(_price, _NormPeriod));
			for (int i = 0; i < count; i++)
			{
				double num = data[i];
				double num2 = data2[i];
				if (num != num2)
				{
					array[i] = (_price[i] - num) / (num2 - num);
				}
				else
				{
					array[i] = 50.0;
				}
			}
			return array;
		}

		// Token: 0x1700026A RID: 618
		public IContext Context
		{
			// Token: 0x0600079D RID: 1949 RVA: 0x000217A0 File Offset: 0x0001F9A0
			get;
			// Token: 0x0600079E RID: 1950 RVA: 0x000217A8 File Offset: 0x0001F9A8
			set;
		}

		// Token: 0x17000269 RID: 617
		[HandlerParameter(true, "10", Min = "1", Max = "30", Step = "1")]
		public int NormalizationPeriod
		{
			// Token: 0x06000799 RID: 1945 RVA: 0x0002158D File Offset: 0x0001F78D
			get;
			// Token: 0x0600079A RID: 1946 RVA: 0x00021595 File Offset: 0x0001F795
			set;
		}
	}
}
