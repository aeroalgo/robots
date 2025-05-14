using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000047 RID: 71
	[HandlerCategory("vvIndicators"), HandlerName("Polarized Fractal Efficiency v3")]
	public class PFEv3 : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000289 RID: 649 RVA: 0x0000C19C File Offset: 0x0000A39C
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("PFEv3", new string[]
			{
				this.PfePeriod.ToString(),
				this.smooth.ToString(),
				src.GetHashCode().ToString()
			}, () => PFEv3.GenPFEv3(src, this.PfePeriod, this.smooth));
		}

		// Token: 0x06000288 RID: 648 RVA: 0x0000C02C File Offset: 0x0000A22C
		public static IList<double> GenPFEv3(IList<double> src, int _pfeperiod, int _smooth)
		{
			int count = src.Count;
			double[] array = new double[count];
			Math.Max(_smooth, 1);
			for (int i = 0; i < count; i++)
			{
				if (i < _pfeperiod)
				{
					array[i] = 0.0;
				}
				else
				{
					double num = Math.Sqrt(Math.Pow(src[i] - src[i - _pfeperiod], 2.0) + Math.Pow((double)_pfeperiod, 2.0));
					double num2 = 0.0;
					for (int j = 1; j <= _pfeperiod; j++)
					{
						num2 += Math.Sqrt(Math.Pow(src[i - j + 1] - src[i - j], 2.0) + 1.0);
					}
					double num3 = 2.0 / (1.0 + (double)_smooth);
					double num4;
					if (src[i] > src[i - _pfeperiod])
					{
						num4 = Math.Round(num / num2 * 100.0);
					}
					else
					{
						num4 = Math.Round(-(num / num2) * 100.0);
					}
					array[i] = array[i - 1] + num3 * (num4 - array[i - 1]);
				}
			}
			return array;
		}

		// Token: 0x170000DB RID: 219
		public IContext Context
		{
			// Token: 0x0600028B RID: 651 RVA: 0x0000C223 File Offset: 0x0000A423
			private get;
			// Token: 0x0600028A RID: 650 RVA: 0x0000C21A File Offset: 0x0000A41A
			set;
		}

		// Token: 0x170000D9 RID: 217
		[HandlerParameter(true, "9", Min = "2", Max = "20", Step = "1")]
		public int PfePeriod
		{
			// Token: 0x06000284 RID: 644 RVA: 0x0000C007 File Offset: 0x0000A207
			get;
			// Token: 0x06000285 RID: 645 RVA: 0x0000C00F File Offset: 0x0000A20F
			set;
		}

		// Token: 0x170000DA RID: 218
		[HandlerParameter(true, "5", Min = "2", Max = "20", Step = "1")]
		public int smooth
		{
			// Token: 0x06000286 RID: 646 RVA: 0x0000C018 File Offset: 0x0000A218
			get;
			// Token: 0x06000287 RID: 647 RVA: 0x0000C020 File Offset: 0x0000A220
			set;
		}
	}
}
