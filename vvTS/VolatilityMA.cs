using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020001A6 RID: 422
	[HandlerCategory("vvAverages"), HandlerName("Volatility (Resetting) MA")]
	public class VolatilityMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D63 RID: 3427 RVA: 0x0003AE64 File Offset: 0x00039064
		public IList<double> Execute(IList<double> price)
		{
			return this.Context.GetData("VoltyMA", new string[]
			{
				this.Lookback.ToString(),
				this.Barrier.ToString(),
				this.InitialPeriod.ToString(),
				price.GetHashCode().ToString()
			}, () => VolatilityMA.GenVolatilityMA(price, this.Lookback, this.InitialPeriod, this.Barrier));
		}

		// Token: 0x06000D62 RID: 3426 RVA: 0x0003ACF0 File Offset: 0x00038EF0
		public static IList<double> GenVolatilityMA(IList<double> src, int _lookback = 100, int _InitialPeriod = 1, double _barrier = 2.0)
		{
			double[] array = new double[src.Count];
			double[] array2 = new double[src.Count];
			int[] array3 = new int[src.Count];
			_lookback = Math.Max(_lookback, 1);
			array2[0] = 0.0;
			for (int i = 1; i < src.Count; i++)
			{
				array2[i] = (src[i] - src[i - 1]) / src[i];
			}
			for (int j = 0; j < src.Count; j++)
			{
				if (j == 0)
				{
					array[j] = src[j];
					array3[j] = 1;
				}
				else
				{
					int num = Math.Min(_lookback, j);
					double num2 = SMA.iSMA_TSLab(array2, j, num);
					double num3 = 0.0;
					for (int k = 0; k < num; k++)
					{
						num3 += Math.Pow(array2[j - k] - num2, 2.0);
					}
					double num4 = Math.Sqrt(num3 / (double)num);
					double value = array2[j] / num4;
					if (Math.Abs(value) > _barrier)
					{
						array3[j] = _InitialPeriod;
					}
					else
					{
						array3[j] = array3[j - 1] + 1;
					}
					array[j] = SMA.iSMA_TSLab(src, j, array3[j]);
				}
			}
			return array;
		}

		// Token: 0x17000459 RID: 1113
		[HandlerParameter(true, "2", Min = "1", Max = "3", Step = "0.5")]
		public double Barrier
		{
			// Token: 0x06000D60 RID: 3424 RVA: 0x0003ACDD File Offset: 0x00038EDD
			get;
			// Token: 0x06000D61 RID: 3425 RVA: 0x0003ACE5 File Offset: 0x00038EE5
			set;
		}

		// Token: 0x1700045A RID: 1114
		public IContext Context
		{
			// Token: 0x06000D64 RID: 3428 RVA: 0x0003AEF4 File Offset: 0x000390F4
			get;
			// Token: 0x06000D65 RID: 3429 RVA: 0x0003AEFC File Offset: 0x000390FC
			set;
		}

		// Token: 0x17000458 RID: 1112
		[HandlerParameter(true, "1", Min = "1", Max = "15", Step = "1")]
		public int InitialPeriod
		{
			// Token: 0x06000D5E RID: 3422 RVA: 0x0003ACCC File Offset: 0x00038ECC
			get;
			// Token: 0x06000D5F RID: 3423 RVA: 0x0003ACD4 File Offset: 0x00038ED4
			set;
		}

		// Token: 0x17000457 RID: 1111
		[HandlerParameter(true, "100", Min = "10", Max = "200", Step = "5")]
		public int Lookback
		{
			// Token: 0x06000D5C RID: 3420 RVA: 0x0003ACBB File Offset: 0x00038EBB
			get;
			// Token: 0x06000D5D RID: 3421 RVA: 0x0003ACC3 File Offset: 0x00038EC3
			set;
		}
	}
}
