using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200010E RID: 270
	[HandlerCategory("vvTrade"), HandlerName("Нормализатор [0..100]")]
	public class Normalizer : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000795 RID: 1941 RVA: 0x00021508 File Offset: 0x0001F708
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("normalized0100", new string[]
			{
				this.NormalizationPeriod.ToString(),
				src.GetHashCode().ToString()
			}, () => Normalizer.GenNormalizer(src, this.NormalizationPeriod, this.Context));
		}

		// Token: 0x06000794 RID: 1940 RVA: 0x00021398 File Offset: 0x0001F598
		public static IList<double> GenNormalizer(IList<double> _price, int _NormPeriod, IContext _ctx)
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
					array[i] = 100.0 * ((_price[i] - num) / (num2 - num));
				}
				else
				{
					array[i] = 50.0;
				}
			}
			return array;
		}

		// Token: 0x17000268 RID: 616
		public IContext Context
		{
			// Token: 0x06000796 RID: 1942 RVA: 0x00021574 File Offset: 0x0001F774
			get;
			// Token: 0x06000797 RID: 1943 RVA: 0x0002157C File Offset: 0x0001F77C
			set;
		}

		// Token: 0x17000267 RID: 615
		[HandlerParameter(true, "10", Min = "1", Max = "30", Step = "1")]
		public int NormalizationPeriod
		{
			// Token: 0x06000792 RID: 1938 RVA: 0x00021356 File Offset: 0x0001F556
			get;
			// Token: 0x06000793 RID: 1939 RVA: 0x0002135E File Offset: 0x0001F55E
			set;
		}
	}
}
