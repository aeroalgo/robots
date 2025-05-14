using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000174 RID: 372
	[HandlerCategory("vvAverages"), HandlerName("FRASMA")]
	public class FRASMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000BBA RID: 3002 RVA: 0x00032778 File Offset: 0x00030978
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("FRASMA", new string[]
			{
				this.SMAperiod.ToString(),
				this.normal_speed.ToString(),
				this.shift.ToString(),
				src.GetHashCode().ToString()
			}, () => FRASMA.GenFRASMA(src, this.SMAperiod, this.normal_speed, this.shift));
		}

		// Token: 0x06000BB9 RID: 3001 RVA: 0x000325D0 File Offset: 0x000307D0
		public static IList<double> GenFRASMA(IList<double> src, int _eperiod, int _normal_speed = 20, int _shift = 0)
		{
			int count = src.Count;
			if (_eperiod < 1)
			{
				_eperiod = 1;
			}
			double[] array = new double[count];
			int num = _eperiod - 1;
			double num2 = Math.Log(2.0);
			for (int i = _eperiod; i < count; i++)
			{
				double num3 = vvSeries.iHighest(src, i, _eperiod);
				double num4 = vvSeries.iLowest(src, i, _eperiod);
				double num5 = 0.0;
				double num6 = 0.0;
				for (int j = 0; j <= num; j++)
				{
					if (num3 - num4 > 0.0)
					{
						double num7 = (src[i - j] - num4) / (num3 - num4);
						if (j > 0)
						{
							num5 += Math.Sqrt(Math.Pow(num7 - num6, 2.0) + 1.0 / Math.Pow((double)_eperiod, 2.0));
						}
						num6 = num7;
					}
				}
				double num8;
				if (num5 > 0.0)
				{
					num8 = 1.0 + (Math.Log(num5) + num2) / Math.Log((double)(2 * num));
				}
				else
				{
					num8 = 0.0;
				}
				double num9 = 1.0 / (2.0 - num8);
				double num10 = num9 / 2.0;
				int period = Convert.ToInt32(Math.Round((double)_normal_speed * num10));
				array[i - _shift] = SMA.iSMA(src, period, i);
			}
			return array;
		}

		// Token: 0x170003DC RID: 988
		public IContext Context
		{
			// Token: 0x06000BBB RID: 3003 RVA: 0x00032808 File Offset: 0x00030A08
			get;
			// Token: 0x06000BBC RID: 3004 RVA: 0x00032810 File Offset: 0x00030A10
			set;
		}

		// Token: 0x170003DA RID: 986
		[HandlerParameter(true, "20", Min = "10", Max = "30", Step = "1")]
		public int normal_speed
		{
			// Token: 0x06000BB5 RID: 2997 RVA: 0x000325AE File Offset: 0x000307AE
			get;
			// Token: 0x06000BB6 RID: 2998 RVA: 0x000325B6 File Offset: 0x000307B6
			set;
		}

		// Token: 0x170003DB RID: 987
		public int shift
		{
			// Token: 0x06000BB7 RID: 2999 RVA: 0x000325BF File Offset: 0x000307BF
			get;
			// Token: 0x06000BB8 RID: 3000 RVA: 0x000325C7 File Offset: 0x000307C7
			set;
		}

		// Token: 0x170003D9 RID: 985
		[HandlerParameter(true, "30", Min = "10", Max = "50", Step = "1")]
		public int SMAperiod
		{
			// Token: 0x06000BB3 RID: 2995 RVA: 0x0003259D File Offset: 0x0003079D
			get;
			// Token: 0x06000BB4 RID: 2996 RVA: 0x000325A5 File Offset: 0x000307A5
			set;
		}
	}
}
