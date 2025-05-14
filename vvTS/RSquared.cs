using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000051 RID: 81
	[HandlerCategory("vvIndicators"), HandlerName("R-Squared")]
	public class RSquared : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x060002E2 RID: 738 RVA: 0x0000DCD7 File Offset: 0x0000BED7
		public IList<double> Execute(IList<double> src)
		{
			return RSquared.GenRSQ(src, this.RSQPeriod);
		}

		// Token: 0x060002E1 RID: 737 RVA: 0x0000DB74 File Offset: 0x0000BD74
		public static IList<double> GenRSQ(IList<double> src, int rsqperiod)
		{
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				if (i < rsqperiod)
				{
					array[i] = 0.0;
				}
				else
				{
					double num = 0.0;
					double num2 = 0.0;
					double num3 = 0.0;
					double num4 = 0.0;
					double num5 = 0.0;
					int num6 = 1;
					for (int j = i - 1; j >= i - rsqperiod; j--)
					{
						double num7 = (double)num6;
						double num8 = src[j];
						num += num7;
						num2 += num8;
						num3 += num7 * num8;
						num4 += Math.Pow(num7, 2.0);
						num5 += Math.Pow(num8, 2.0);
						num6++;
					}
					double num9 = Math.Pow(num, 2.0);
					double num10 = Math.Pow(num2, 2.0);
					double num11 = Math.Sqrt(((double)rsqperiod * num4 - num9) * ((double)rsqperiod * num5 - num10));
					if (num11 == 0.0)
					{
						array[i] = 0.0;
					}
					else
					{
						double x = ((double)rsqperiod * num3 - num * num2) / num11;
						array[i] = Math.Pow(x, 2.0);
					}
				}
			}
			return array;
		}

		// Token: 0x170000F9 RID: 249
		[HandlerParameter(true, "10", Min = "5", Max = "100", Step = "1")]
		public int RSQPeriod
		{
			// Token: 0x060002DF RID: 735 RVA: 0x0000DB60 File Offset: 0x0000BD60
			get;
			// Token: 0x060002E0 RID: 736 RVA: 0x0000DB68 File Offset: 0x0000BD68
			set;
		}
	}
}
